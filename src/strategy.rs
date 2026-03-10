// ─── Partie 3 : Stratégie de déplacement ─────────────────────────────────────
//
// Objectif : définir un trait Strategy et l'utiliser via Box<dyn Strategy>
// pour choisir le prochain mouvement du bot à chaque tick.
//
// Concepts exercés : dyn Trait, Box<dyn Strategy>, Send, dispatch dynamique.
//
// ─────────────────────────────────────────────────────────────────────────────

// TODO: Importer les types nécessaires de state.rs
use crate::state::{GameState, ResourceInfo};
// TODO: Définir le trait Strategy.
//
// Le trait doit :
//   - Être object-safe (pas de generics dans les méthodes)
//   - Être Send (pour pouvoir être utilisé dans un contexte multi-thread)
//   - Avoir une méthode next_move qui retourne un déplacement optionnel
//
pub trait Strategy: Send {
    fn next_move(&self, state: &GameState) -> Option<(i8, i8)>;
}

// TODO: Implémenter NearestResourceStrategy.
//
// Cette stratégie se dirige vers la ressource la plus proche (distance de Manhattan).
//
pub struct NearestResourceStrategy;

impl Strategy for NearestResourceStrategy {
    fn next_move(&self, state: &GameState) -> Option<(i8, i8)> {
       let proche=state.resources
        .iter()
        .min_by_key(|r|{
            let dx = (r.x as i32 - state.position.0 as i32).abs();
            let dy=(r.y as i32 - state.position.1 as i32).abs();
            dx + dy 
        });
        let Some(ressource) = proche else { return None; };
        let dx = if ressource.x > state.position.0 { 1i8 } else if  ressource.x < state.position.0 { -1i8 } else {0i8};
        let dy = if ressource.y > state.position.1 { 1i8 } else if  ressource.y < state.position.1 { -1i8 } else {0i8};
        // 2. Calculer la direction (dx, dy) vers cette ressource :
        //    - Si resource.x > position.x → dx = 1
        //    - Si resource.x < position.x → dx = -1
        //    - Sinon dx = 0
        //    - Idem pour dy
        //
        //    Indice : utilisez i16 pour les calculs puis .signum() puis cast en i8
        //
        // 3. Retourner Some((dx, dy)), ou None si aucune ressource
        Some((dx,dy))
    }
}

// ─── BONUS : Implémenter d'autres stratégies ────────────────────────────────
//
// Exemples :
//   - RandomStrategy : mouvement aléatoire
//   - FleeStrategy : s'éloigne des autres agents
//   - HybridStrategy : combine plusieurs stratégies
//
// Utilisation dans main.rs :
//   let strategy: Box<dyn Strategy> = Box::new(NearestResourceStrategy);
//
// On peut changer de stratégie sans modifier le reste du code grâce au
// dispatch dynamique (Box<dyn Strategy>).
use rand::Rng;

// ─── RandomStrategy ───────────────────────────────────────────
pub struct RandomStrategy;

impl Strategy for RandomStrategy {
    fn next_move(&self, _state: &GameState) -> Option<(i8, i8)> {
        let mut rng = rand::thread_rng();
        let dx = rng.gen_range(-1i8..=1);
        let dy = rng.gen_range(-1i8..=1);
        Some((dx, dy))
    }
}

// ─── FleeStrategy — s'éloigne des autres agents ───────────────
pub struct FleeStrategy;

impl Strategy for FleeStrategy {
    fn next_move(&self, state: &GameState) -> Option<(i8, i8)> {
        // Trouver l'agent le plus proche (pas nous)
        let nearest = state.agents
            .iter()
            .filter(|a| a.id != state.agent_id)
            .min_by_key(|a| {
                (a.x as i32 - state.position.0 as i32).abs()
                + (a.y as i32 - state.position.1 as i32).abs()
            })?;

        // S'éloigner → inverser la direction
        let dx = -(nearest.x as i16 - state.position.0 as i16).signum() as i8;
        let dy = -(nearest.y as i16 - state.position.1 as i16).signum() as i8;
        Some((dx, dy))
    }
}

// ─── HybridStrategy — fuit si danger, sinon mine ──────────────
pub struct HybridStrategy {
    pub flee_distance: i32,  // distance à partir de laquelle on fuit
}

impl Strategy for HybridStrategy {
    fn next_move(&self, state: &GameState) -> Option<(i8, i8)> {
        // Vérifier si un agent est trop proche
        let too_close = state.agents
            .iter()
            .filter(|a| a.id != state.agent_id)
            .any(|a| {
                let dist = (a.x as i32 - state.position.0 as i32).abs()
                         + (a.y as i32 - state.position.1 as i32).abs();
                dist <= self.flee_distance
            });

        if too_close {
            FleeStrategy.next_move(state)
        } else {
            NearestResourceStrategy.next_move(state)
        }
    }
}
