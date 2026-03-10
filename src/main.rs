#![allow(dead_code)]

mod miner;
mod pow;
mod protocol;
mod state;
mod strategy;
use crate::strategy::Strategy;

use std::thread;
use std::time::Duration;

use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use uuid::Uuid;

use protocol::{ClientMsg, ServerMsg};

const SERVER_URL: &str = "ws://127.0.0.1:4004/ws";
const TEAM_NAME: &str = "Wiwi";
const AGENT_NAME: &str = "bot_1";
const NUM_MINERS: usize = 4;

type WsStream = WebSocket<MaybeTlsStream<std::net::TcpStream>>;

fn read_server_msg(ws: &mut WsStream) -> Option<ServerMsg> {
    match ws.read() {
        Ok(Message::Text(text)) => {
            match serde_json::from_str(&text) {
                Ok(msg) => Some(msg),
                Err(_) => None,
            }
        }
        Ok(_) => None,
        Err(e) => {
            eprintln!("[!] Erreur WS lecture : {e}");
            None
        }
    }
}

fn send_client_msg(ws: &mut WsStream, msg: &ClientMsg) {
    let json = serde_json::to_string(msg).expect("sérialisation échouée");
    ws.send(Message::Text(json.into())).expect("envoi WS échoué");
}

fn main() {
    println!("[*] Connexion à {SERVER_URL}...");
    let (mut ws, _response) = connect(SERVER_URL).expect("impossible de se connecter au serveur");
    println!("[*] Connecté !");

    // ── Attendre le Hello ─────────────────────────────────────────────────
    let agent_id: Uuid = match read_server_msg(&mut ws) {
        Some(ServerMsg::Hello { agent_id, tick_ms }) => {
            println!("[*] Hello reçu : agent_id={agent_id}, tick={tick_ms}ms");
            agent_id
        }
        other => panic!("premier message inattendu : {other:?}"),
    };

    // ── S'enregistrer ─────────────────────────────────────────────────────
    send_client_msg(
        &mut ws,
        &ClientMsg::Register {
            team: TEAM_NAME.into(),
            name: AGENT_NAME.into(),
        },
    );
    println!("[*] Enregistré en tant que {AGENT_NAME} (équipe {TEAM_NAME})");

    // ── Init ──────────────────────────────────────────────────────────────
    let shared_state = state::new_shared_state(agent_id);
    let miner_pool = miner::MinerPool::new(NUM_MINERS);
    let strategy: Box<dyn strategy::Strategy> = Box::new(strategy::NearestResourceStrategy);

    let mut last_pos = (0u16, 0u16);
    let mut stuck_count = 0u32;

    // ── Boucle principale ─────────────────────────────────────────────────
    loop {
        // 1. Lire UN message du serveur (bloquant)
        if let Some(msg) = read_server_msg(&mut ws) {
            shared_state.lock().unwrap().update(&msg);
            match msg {
               ServerMsg::PowChallenge { tick, seed, resource_id, target_bits, .. } => {
                    println!("[*] PowChallenge reçu ! resource={resource_id} bits={target_bits}");
                    miner_pool.submit(miner::MineRequest {
                        seed, tick, resource_id, agent_id, target_bits,
                    });
                }
                ServerMsg::Win { team } => {
                    println!("[*] Victoire de l'équipe {team} !");
                    return;
                }
                _ => {}
            }
        }

        // 2. Solutions minées → envoyer PowSubmit
        while let Some(result) = miner_pool.try_recv() {
            println!("[*] Nonce trouvé pour {} !", result.resource_id);
            send_client_msg(&mut ws, &ClientMsg::PowSubmit {
                tick: result.tick,
                resource_id: result.resource_id,
                nonce: result.nonce,
            });
        }

        // 3. Mouvement avec détection de blocage
        let movement = {
            let state = shared_state.lock().unwrap();
            let pos = state.position;

            if pos == last_pos {
                stuck_count += 1;
            } else {
                stuck_count = 0;
                last_pos = pos;
            }

            // Si bloqué depuis plus de 3 ticks → direction aléatoire
            if stuck_count > 3 {
                strategy::RandomStrategy.next_move(&state)
            } else {
                strategy.next_move(&state)
            }
        };

        if let Some((dx, dy)) = movement {
            send_client_msg(&mut ws, &ClientMsg::Move { dx, dy });
        }

        thread::sleep(Duration::from_millis(50));
    }
}