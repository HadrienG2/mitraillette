mod choix;
mod combinaison;
mod stats;

use crate::{
    combinaison::VALEUR_MIN_DE,
    stats::Stats,
};


// Type flottant utilisé pour les probabilités et les espérances
type Flottant = f32;

// Type destiné à stocker des valeurs de combinaisons, de mises, de scores...
type Valeur = u16;

// Nombre de dés maximum qu'on peut lancer
const NB_DES_TOT : usize = 6;

// Nombre de faces par dé
const NB_FACES : usize = 6;

// Nombre maximal de relances considéré, utile pour éviter d'explorer des
// régions trop improbables de l'arbre des possibles
const NB_RELANCES_MAX : usize = 11;

// Score maximal atteignable. On doit l'atteindre exactement pour terminer.
const SCORE_MAX : Valeur = 10000;

// Mises pour lesquelles on estime les espérances de gain à chaque nombre de dés
const MISES : [Valeur; 23] = [0, 50, 100, 150, 200, 250, 300, 350, 400, 450,
                              500, 700, 950, 1000, 1300, 1600, 2000, 2300, 2600,
                              2850, 2900, 9250, 9300];

// Toutes les combinaisons (score, mise, nb de dés) ne sont pas vraisemblables.
// Par exemple, si on lance un seul dé, on a nécessairement accumulé 250 points,
// et si on a gagné, on ne relance pas
fn jet_impossible(score: Valeur, nb_des: usize, mise: Valeur) -> bool {
    score + mise >= SCORE_MAX || if nb_des < NB_DES_TOT {
        // Si on n'a pas tous les dés, on a tiré au moins 50 points des autres
        mise < (NB_DES_TOT - nb_des) as Valeur * VALEUR_MIN_DE
    } else {
        // Si on a tous les dés, on est au début ou on a pris 6 dés avant
        mise > 0 && mise < NB_DES_TOT as Valeur * VALEUR_MIN_DE
    }
}


fn main() {
    // Tout d'abord, on explore les résultats de jets possibles...
    let stats = Stats::new();

    // Ensuite, on tabule les espérances de gain à score nul
    println!("\n=== ESPERANCES DE GAIN A SCORE NUL ===");

    // On tabule les espérances à ce nombre de relances
    for nb_des in 1..=NB_DES_TOT {
        println!("\nEn lançant {} dés:", nb_des);

        // Puis, pour chaque mise considérée...
        for &mise in MISES.iter() {
            // On rejette les situations impossibles
            if jet_impossible(0, nb_des, mise) { continue; }

            // ...et sinon, on affiche ce qu'on gagne à (re)lancer en moyenne
            let gain_moyen = stats.gain_moyen(0, nb_des, mise);
            println!("- Mise {}: {:+}", mise, gain_moyen);
        }
    }
    println!();

    // Ensuite, on tabule les espérances de gain à score nul
    println!("\n=== PROBABILITE DE GAGNER CE TOUR-CI ===\n");

    for score in (8000..10000).rev().filter(|s| s % 50 == 0) {
        let proba = stats.proba_fin(score, 6, 0, NB_RELANCES_MAX);
        println!("Score {}, 6 dés sans mise: {}", score, proba);
    }
    println!();

    // TODO: Etudier atterissages en partant d'un nombre de dés différent, d'une
    //       mise différente, en plusieurs tours...

    // TODO: Faire des combats de robots
}