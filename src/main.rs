mod choix;
mod combinaison;
mod stats;

use crate::{
    combinaison::{Valeur, VALEUR_MIN_DE},
    stats::StatsDes,
};


// On va compter les probas sur 64-bit au cas où, on diminuera si besoin
type Flottant = f32;

// Nombre de dés maximum qu'on peut lancer
const NB_DES_TOT : usize = 6;

// Nombre de faces par dé
const NB_FACES : usize = 6;

// Mises pour lesquelles on estime les espérances de gain à chaque nombre de dés
const NB_MISES : usize = 24;
const MISES : [Valeur; NB_MISES] = [0, 50, 100, 150, 200, 250, 300, 350, 400,
                                    450, 500, 700, 900, 950, 1000, 1300, 1600,
                                    2000, 2300, 2600, 2800, 2850, 2900, 10000];

// Toutes les combinaisons (mise, nb de dés) ne sont pas possibles. Par exemple,
// si on lance à un dé, on a nécessairement accumulé au moins 5x50 = 250 points
fn mise_impossible(nb_des: usize, mise: Valeur) -> bool {
    if nb_des < NB_DES_TOT {
        // Si on n'a pas tous les dés, on a tiré au moins 50 points des autres
        mise < (NB_DES_TOT - nb_des) as Valeur * VALEUR_MIN_DE
    } else {
        // Si on a tous les dés, on est au début ou on a pris 6 dés avant
        mise > 0 && mise < NB_DES_TOT as Valeur * VALEUR_MIN_DE
    }
}


fn main() {
    // Tout d'abord, on explore les résultats de jets possibles...
    println!("Exploration de la combinatoire des dés...");
    let stats_des = StatsDes::new();

    // On tabule les espérances à ce nombre de relances
    for nb_des in 1..=NB_DES_TOT {
        println!("\nEspérances de gain à {} dés:", nb_des);

        // Puis, pour chaque mise considérée...
        for &mise in MISES.iter() {
            // On rejette les combinaisons (mise, nb de dés) impossibles
            if mise_impossible(nb_des, mise) { continue; }

            // ...et sinon, on affiche le résultat brut
            let esperance_lancer = stats_des.esperance(nb_des, mise);
            let gain_moyen = esperance_lancer - mise as Flottant;
            println!("- Mise {}: {:+}", mise, gain_moyen);
        }
    }
    println!();

    // TODO: Etudier l'atterissage
    //
    //       On va commencer par déterminer à partir de quel score on a une
    //       chance maximale de finir en un tour. Après, on pourra considérer
    //       la fin en plusieurs tours, et ainsi remonter de proche en proche
    //       à une stratégie optimale sur l'ensemble de la partie, à nombre de
    //       points fini, si on explose pas le temps de calcul avant.

    // TODO: Etudier les autres effets de score fini
    // TODO: Faire des combats de robots
}