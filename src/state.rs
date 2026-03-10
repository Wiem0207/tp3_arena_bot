// ─── Partie 1 : État partagé ─────────────────────────────────────────────────
//
// Objectif : définir un GameState protégé par Arc<Mutex<>> pour être partagé
// entre le thread lecteur WS et le thread principal.
//
// Concepts exercés : Arc, Mutex, struct, closures.
//
// ─────────────────────────────────────────────────────────────────────────────

// Ces imports seront utilisés dans votre implémentation.
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]
use uuid::Uuid;

#[allow(unused_imports)]
use crate::protocol::ServerMsg;

/// Information sur une ressource (challenge de minage) active sur la carte.
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub resource_id: Uuid,
    pub x: u16,
    pub y: u16,
    pub expires_at: u64,
}

/// Information sur un agent visible sur la carte.
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: Uuid,
    pub name: String,
    pub team: String,
    pub score: u32,
    pub x: u16,
    pub y: u16,
}

pub struct GameState{
    pub agent_id: Uuid,
    pub tick:u64, 
    pub position:(u16,u16),
    pub map_size:(u16,u16),
    pub goal: u32,
    pub obstacles:  Vec<(u16,u16)>,
    pub resources:  Vec<ResourceInfo>,
    pub agents: Vec<AgentInfo> ,
    pub team_score: HashMap<String,u32>,
}
impl GameState{
    pub fn new(agent_id:Uuid)->Self{
        Self {
            agent_id,
            tick:0,
            position:(0,0),
            map_size:(0,0),
            goal:0,
            obstacles: Vec::new() ,
            resources : Vec::new(),
            agents : Vec::new(),
            team_score: HashMap::new()
        }
    }
    pub fn update(&mut self, msg: &ServerMsg){
        match msg {
            ServerMsg::State { tick, width, height, goal, obstacles, resources, agents }=> {
            self.tick=*tick ;
            self.map_size=(*width,*height) ;
            self.goal=*goal ; 
            self.obstacles=obstacles.clone() ;
            self.agents = agents
            .iter()
            .map(|(id, name, team, score, x, y)| {
                if *id == self.agent_id {
                    self.position = (*x, *y);
                }
                AgentInfo {
                    id: *id,
                    name: name.clone(),
                    team: team.clone(),
                    score: *score,
                    x: *x,
                    y: *y,
                }
            })
            .collect();
           self.resources = resources
            .iter()
            .map(|(id, x, y, expires_at, _points)| ResourceInfo {
                resource_id: *id,
                x: *x,
                y: *y,
                expires_at: *expires_at,
            })
            .collect();
            self.team_score.clear() ;
            for (id, name, team, score, x, y) in agents {
                self.team_score.insert(team.clone(),*score) ;
            }
            }
            ServerMsg::PowResult{resource_id,..}=>{
                self.resources.retain(|r| r.resource_id != *resource_id);
            }
            _ => {} 
    }
}
}

// definir un alias est comme ça : 
pub type SharedState=Arc<Mutex<GameState>> ;
pub fn new_shared_state(agent_id: Uuid) -> SharedState {
    Arc::new(Mutex::new(GameState::new(agent_id)))
}
