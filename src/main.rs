mod choix;
mod combinaison;

use crate::combinaison::{Combinaison, Valeur};

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt::{self, Debug},
};


// Nombre de dés maximum qu'on peut lancer
const NB_DES_TOT : usize = 6;

// Nombre de faces par dé
const NB_FACES : usize = 6;

// Nombre maximal de relances qu'on peut considérer
const PROFONDEUR_MAX : usize = 30;

// Mises pour lesquelles on estime les espérances de gain à chaque nombre de dés
const NB_MISES : usize = 15;
const MISES : [Valeur; NB_MISES] = [0, 50, 100, 150, 200, 250, 300, 350, 400,
                                    950, 1000, 2000, 2850, 2900, 10000];

// Toutes les combinaisons (mise, nb de dés) ne sont pas possibles. Par exemple,
// si on lance à un dé, on a nécessairement accumulé au moins 5x50 = 250 points
fn mise_impossible(mise: Valeur, nb_des: usize) -> bool {
    let mise = mise as usize;
    match nb_des {
        NB_DES_TOT => mise > 0 && mise < NB_DES_TOT * 50,
        _ => mise < (NB_DES_TOT - nb_des) * 50,
    }
}

// On va compter les probas sur 64-bit au cas où, on diminuera si besoin
type Flottant = f32;

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face, si on tire des combinaisons
    stats_choix: Vec<StatsChoix>,

    // Espérance pour une mise donnée au dernier nombre de relances considéré
    esperance_actuelle: Cell<[Flottant; NB_MISES]>,
}

// L'un dex choix face auxquels un jet de dés peut nous placer
struct StatsChoix {
    // Combinaisons entre lesquels il faut choisir
    choix: Vec<Possibilite>,

    // Probabilité qu'on a de faire face à ce choix
    proba: Flottant,

    // On garde en cache l'espérance de la stratégie optimale pour un nombre
    // de relances <= N donné et une certaine mise initiale
    esperance_max: RefCell<HashMap<(usize, Valeur), Flottant>>,
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
                        esperance_max: RefCell::new(HashMap::new()),
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
            esperance_actuelle: Cell::new([0.; NB_MISES]),
        });

        println!();
    }

    // On calcule l'espérance de gain en s'autorisant à relancer les dés au plus
    // N fois avec N croissant (initialement 0: on ne relance jamais).
    for num_relances in 0..=PROFONDEUR_MAX {
        println!("=== JETS AVEC <={} RELANCES ===\n", num_relances);
        let mut continuer = false;

        // On fait ça pour chaque nombre de dés...
        for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
            let nb_des = idx_nb_des + 1;
            println!("Espérances à {} dés:", nb_des);

            // ...et pour chaque mise initiale considérée
            for (idx_mise, &mise) in MISES.iter().enumerate() {
                // On rejette les combinaisons (mise, nb de dés) impossibles
                if mise_impossible(mise, nb_des) { continue; }

                // On calcule l'espérance de gain récursivement (ci-dessous)
                let esperance_avec_relances = iterer_esperance(num_relances, &stats_jets[..], stats, mise);

                // On affiche le résultat brut
                let esperance_gain = esperance_avec_relances - mise as Flottant;
                print!("- Mise {}: Esperance >= {} (Gain moyen >= {:+}, ",
                       mise, esperance_avec_relances, esperance_gain);

                // On regarde l'évolution par rapport au nombre de lancer max
                // considéré précédemment
                let mut esperances = stats.esperance_actuelle.get();
                let delta = esperance_avec_relances - esperances[idx_mise];
                esperances[idx_mise] = esperance_avec_relances;
                stats.esperance_actuelle.set(esperances);

                // Cette évolution doit etre positive, on continue d'itérer tant
                // qu'elle est non nulle (précision flottant pas atteinte)
                assert!(delta >= 0.);
                if delta > 0. {
                    println!("Delta = {:+})", delta);
                    continuer = true;
                } else {
                    println!("STABLE)");
                }
            }

            println!();
        }

        if !continuer { break; }
    }
    // TODO: Quand on a fini d'itérer sur l'espérance, on peut jeter les
    //       espérances max à profondeur inférieures, elles ne serviront plus.

    // TODO: Etudier l'atterissage

    // TODO: Etudier les autres effets de score fini
    // TODO: Faire des combats de robots
}

// Calcule de l'espérance de gain pour une stratégie où on relance les dés
// jusqu'à N fois en partant d'une certaine mise et d'un certain nombre de dés
//
// NOTE: Pour une profondeur N donnée, le code suppose que toutes les
// profondeurs 0 <= k < N précédentes ont déjà été étudiées.
//
fn iterer_esperance(num_relances: usize, stats_jets: &[StatsJet], stats_jet_actuel: &StatsJet, mise: Valeur) -> Flottant {
    // Le but est de déterminer une espérance de gain pour un certain lancer
    let mut esperance_lancer = 0.;

    // On passe en revue tous les résultats de lancers gagnants
    for stats_choix in stats_jet_actuel.stats_choix.iter() {
        // Est-ce que, par chance, j'ai déjà étudié ce cas à une itération
        // précédente du calcul en cours?
        if let Some(esperance_max) = stats_choix.esperance_max.borrow().get(&(num_relances, mise)) {
            esperance_lancer += esperance_max * stats_choix.proba;
            continue;
        }

        // Après avoir obtenu des combinaisons, il faut choisir la stratégie
        // optimale entre empocher l'une des combinaisons disponibles (la plus
        // chère de préférence) et relancer les dés N fois. Les cas où on
        // relance les dés <N fois ont déjà été traités, c'est donc seulement
        // le cas où on relance exactement N fois qui nous intéresse.
        let mut esperance_max = if num_relances == 0 {
            // A zéro relances, le cas initial c'est celui où on ne lance pas
            mise as Flottant
        } else {
            // A N>0 relances, c'est celui où on lance N-1 fois
            //
            // NOTE: C'est ce point-là du code qu'il faudrait modifier pour ne
            //       plus avoir besoin de lancer iterer_esperance à toutes les
            //       profondeurs précédentes au préalable.
            //
            stats_choix.esperance_max.borrow()[&(num_relances - 1, mise)]
        };

        // On étudie ensuite toutes les possibilités de relance une à une, dans
        // une stratégie où on relance toujours jusqu'à la profondeur N (les
        // profondeurs <N ont été déjà examinées et intégrées à esperance_max)
        for poss in stats_choix.choix.iter() {
            // Est-ce qu'on a atteint le nombre de relances désiré?
            let esperance_relance = if num_relances == 0 {
                // Si oui, on prend les gains systématiquement.
                (mise + poss.valeur) as Flottant
            } else {
                // Sinon, on ajoute la valeur à la mise et relance au nouveau
                // nombre de dés, en décrémentant le budget relance
                iterer_esperance(num_relances - 1,
                                 stats_jets,
                                 &stats_jets[poss.nb_des_relance - 1],
                                 mise + poss.valeur)
            };

            // Au fil du temps, on garde une trace de la stratégie considérée
            // (selon les combinaisons, et le nombre de relance) qui rapporte le
            // plus gros, selon notre critère de l'espérance de gain.
            esperance_max = esperance_max.max(esperance_relance);
        }

        // L'espérance de gain la plus forte mesurée est mise en cache, ce qui
        // permet d'étudier les cas où on relance de 0 à N fois en étudiant
        // juste le cas où on relance exactement N fois.
        assert_eq!(stats_choix.esperance_max.borrow_mut()
                       .insert((num_relances, mise), esperance_max),
                   None);

        // Et en intégrant les espérances de gain des stratégies optimales
        // (à cette profondeur de relance) sur tous les lancers de dés
        // possibles, on en déduit l'espérance de gain pour une stratégie
        // optimale à <=N relances quel que soit le résultat des dés.
        esperance_lancer += esperance_max * stats_choix.proba;
    }

    // On retourne ce résultat à l'appelant
    esperance_lancer
}