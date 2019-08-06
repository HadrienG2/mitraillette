mod choix;
mod combinaison;

use crate::combinaison::{Combinaison, Valeur};

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Debug},
};


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
fn mise_impossible(mise: Valeur, nb_des: usize) -> bool {
    let mise = mise as usize;
    if nb_des < NB_DES_TOT {
        // Si on n'a pas tous les dés, on a tiré au moins 50 points des autres
        mise < (NB_DES_TOT - nb_des) * 50
    } else {
        // Si on a tous les dés, on est au début ou on a pris 6 dés avant
        mise > 0 && mise < NB_DES_TOT * 50
    }
}

// On va compter les probas sur 64-bit au cas où, on diminuera si besoin
type Flottant = f32;

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face, si on tire des combinaisons
    stats_choix: Vec<StatsChoix>,

    // On garde en cache l'espérance de gain pour un nombre de relances <= N
    // donné et une certaine mise initiale.
    esperance: RefCell<HashMap<(usize, Valeur), Flottant>>,
}

// L'un dex choix face auxquels un jet de dés peut nous placer
struct StatsChoix {
    // Combinaisons entre lesquels il faut choisir
    choix: Vec<Possibilite>,

    // Probabilité qu'on a de faire face à ce choix
    proba: Flottant,
}

// L'une des possibilités entre lesquelles il faut choisir
struct Possibilite {
    // Combinaison qu'on décide ou non de choisir
    comb: Combinaison,

    // Valeur de cette combinaison
    valeur: Valeur,

    // Nombre de dés avec lequel on peut relancer ensuite
    nb_des_relance: usize,
}

impl Debug for Possibilite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?} ({}pt, relance {}d)",
               self.comb, self.valeur, self.nb_des_relance)
    }
}


fn main() {
    println!("\n=== ETUDE DES JETS SANS RELANCES ===\n");

    // Tout d'abord, pour chaque nombre de dés, on énumère les tiragess et
    // on en déduit la probabilité de perdre et les choix auxquels on peut faire
    // face en cas de réussite, avec la probabilité de chacun.
    let mut stats_jets = Vec::new();

    // On étudie des lancers de 1 à 6 dés
    for nb_des in 1..=NB_DES_TOT {
        // On énumère les choix de combinaisons face auxquels on peut se
        // retrouver en lançant ce nombre de dés, et avec quelle probabilité.
        let mut choix_et_probas = choix::enumerer_choix(nb_des);

        // On met à part le cas perdant, car il est spécial à plusieurs égards
        // (on perd la mise précédente, on ne peut pas choisir de continuer)
        let proba_perte = choix_et_probas.remove(&[][..]).unwrap();

        // Pour les autres choix, on note quelques compléments
        let stats_choix =
            choix_et_probas.into_iter()
                .map(|(choix, proba)| {
                    // On annote chaque combinaison de chaque choix avec sa
                    // valeur et le nombre dés dont on dispose en cas de relance
                    let choix = choix.into_iter()
                        .map(|comb| {
                            let valeur = comb.valeur();
                            let des_restants = nb_des - comb.nb_des();
                            let nb_des_relance = if des_restants == 0 {
                                NB_DES_TOT
                            } else {
                                des_restants
                            };
                            Possibilite {
                                comb,
                                valeur,
                                nb_des_relance,
                            }
                        }).collect::<Vec<_>>();

                    // On transforme notre comptage en probabilité
                    StatsChoix {
                        choix,
                        proba,
                    }
                }).collect::<Vec<StatsChoix>>();

        // La probabilité de gagner est plus utile que celle de perdre, car elle
        // dit quelle proportion de la mise on garde en moyenne en relançant
        let proba_gain = 1. - proba_perte;
        println!("Probabilité de gagner à {} dés: {}", nb_des, proba_gain);

        // Nous gardons de côté ces calculs, on a besoin de les avoir effectués
        // pour tous les nombres de dés avant d'aller plus loin.
        stats_jets.push(StatsJet {
            stats_choix,
            esperance: RefCell::new(HashMap::new()),
        });

        println!();
    }

    println!("=== ETUDE DES RELANCES ===\n");

    println!("Choix du nombre de relances...");

    // On ajuste le nombre de relances en se basant sur la convergence de
    // l'espérance d'un lancer de 6 dés à mise initiale nulle (soit l'espérance
    // d'un tour, mais aussi le cas où on est le plus encouragé à relancer).
    let stats_init = &stats_jets[NB_DES_TOT-1];
    let mut num_relances = 0;
    let mut ancienne_esperance = 0.;
    loop {
        let esperance = esperance(num_relances, &stats_jets[..], stats_init, 0);
        println!("- Avec {} relances, l'espérance d'un tour est {}", num_relances, esperance);
        assert!(esperance >= ancienne_esperance);
        if esperance == ancienne_esperance { break; }
        ancienne_esperance = esperance;
        num_relances += 1;
    }

    // On résume les résultats...
    println!("Convergence atteinte avec {} relances!", num_relances);
    println!();

    // ...pour chaque nombre de dés...
    for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
        let nb_des = idx_nb_des + 1;
        println!("Espérances de gain à {} dés:", nb_des);

        // Puis, pour chaque mise considérée...
        for &mise in MISES.iter() {
            // On rejette les combinaisons (mise, nb de dés) impossibles
            if mise_impossible(mise, nb_des) { continue; }

            // ...et sinon, on affiche le résultat brut
            let esperance_lancer = esperance(num_relances,
                                             &stats_jets[..],
                                             stats,
                                             mise);
            let gain_moyen = esperance_lancer - mise as Flottant;
            println!("- Mise {}: {:+}", mise, gain_moyen);
        }
        println!();
    }

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

// Calcule de l'espérance de gain pour une stratégie où on relance les dés
// jusqu'à N fois en partant d'une certaine mise et d'un certain nombre de dés
fn esperance(num_relances: usize, stats_jets: &[StatsJet], stats_jet_actuel: &StatsJet, mise: Valeur) -> Flottant {
    // Est-ce que, par chance, j'ai déjà étudié ce cas précédemment?
    if let Some(&esperance_lancer) = stats_jet_actuel.esperance.borrow().get(&(num_relances, mise)) {
        return esperance_lancer;
    }

    // Le but est de déterminer une espérance de gain pour un certain lancer
    let mut esperance_lancer = 0.;

    // On passe en revue tous les résultats de lancers gagnants
    for stats_choix in stats_jet_actuel.stats_choix.iter() {
        // On peut empocher la combinaison de valeur maximale...
        let valeur_max = stats_choix.choix.iter()
                                          .map(|poss| poss.valeur)
                                          .max()
                                          .unwrap();

        // ...ou bien on peut relancer, de 1 à num_relances fois. Parmi ces
        // options, on cherche celle qui maximise l'espérance de gain.
        let mut esperance_max = (mise + valeur_max) as Flottant;

        // Pour traiter les relances, il suffit de considérer chaque combinaison
        // qu'on peut empocher, l'ajouter à la mise, et faire une récursion avec
        // le nouveau nombre de dés et un budget relance réduit
        for num_relances in 1..=num_relances {
            for poss in stats_choix.choix.iter() {
                let esperance =
                    esperance(num_relances - 1,
                              stats_jets,
                              &stats_jets[poss.nb_des_relance - 1],
                              mise + poss.valeur);
                esperance_max = esperance_max.max(esperance);
            }
        }

        // A la fin, on pondère cette espérance maximale par la probabilité de
        // faire face au choix qu'on a considéré.
        esperance_lancer += esperance_max * stats_choix.proba;
    }

    // On met en cache ce résultat
    assert_eq!(stats_jet_actuel.esperance.borrow_mut()
                   .insert((num_relances, mise), esperance_lancer),
               None);

    // On retourne ce résultat à l'appelant
    esperance_lancer
}