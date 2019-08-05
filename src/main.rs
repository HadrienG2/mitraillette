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

// Soldes pour lesquels on estime les espérances de gain
const NB_SOLDES : usize = 15;
const SOLDES : [Valeur; NB_SOLDES] = [0, 50, 100, 150, 200, 250, 300, 350, 400,
                                      950, 1000, 2000, 2850, 2900, 10000];

// ...mais tous les soldes ne sont pas dignes d'être affichés
fn solde_impossible(solde: Valeur, nb_des: usize) -> bool {
    let solde = solde as usize;
    match nb_des {
        NB_DES_TOT => solde > 0 && solde < NB_DES_TOT * 50,
        _ => solde < (NB_DES_TOT - nb_des) * 50,
    }
}

// On va compter les probas sur 64-bit au cas où, on diminuera si besoin
type Flottant = f32;

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face, si on tire des combinaisons
    stats_choix: Vec<StatsChoix>,

    // Espérance à la dernière profondeur de relance calculée
    esperance_actuelle: Cell<[Flottant; NB_SOLDES]>,
}

// L'un dex choix face auxquels un jet de dés peut nous placer
struct StatsChoix {
    // Combinaisons entre lesquels il faut choisir
    choix: Vec<Possibilite>,

    // Probabilité qu'on a de faire face à ce choix
    proba: Flottant,

    // On garde en cache l'espérance de la stratégie optimale pour un nombre
    // de relances <= N donné et un certain solde initial avant de jouer
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
                                6
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

        // La probabilité de gagner est plus utile que la probabilité de perdre,
        // elle dit quelle proportion du solde on garde en moyenne en relançant
        let proba_gain = 1. - proba_perte;
        println!("Probabilité de gagner à {} dés: {}", nb_des, proba_gain);

        // Nous gardons de côté ces calculs, on a besoin de les avoir effectués
        // pour tous les nombres de dés avant d'aller plus loin.
        stats_jets.push(StatsJet {
            stats_choix,
            esperance_actuelle: Cell::new([0.; NB_SOLDES]),
        });

        println!();
    }

    // Maintenant, on explore le graphe de décision avec un nombre croissant
    // de relances autorisées (...et un temps de calcul qui explose)
    for profondeur in 0..=PROFONDEUR_MAX {
        println!("=== JETS AVEC <={} RELANCES ===\n", profondeur);
        let mut continuer = false;

        // Maintenant, on reprend le calcul en autorisant une relance
        for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
            let nb_des = idx_nb_des + 1;
            println!("Espérances à {} dés:", nb_des);

            // Comme précédemment, on considère différents soldes de départ
            for (idx_solde, &solde_initial) in SOLDES.iter().enumerate() {
                // On calcule l'espérance à profondeur N récursivement (cf ci-dessous)
                let esperance_avec_relances = iterer_esperance(profondeur, &stats_jets[..], stats, solde_initial);

                // On vérifie que les espérances calculées sont croissantes
                let mut esperances = stats.esperance_actuelle.get();
                let delta = esperance_avec_relances - esperances[idx_solde];
                assert!(delta >= 0.);
                if delta > 0. { continuer = true; }
                esperances[idx_solde] = esperance_avec_relances;
                stats.esperance_actuelle.set(esperances);

                // Si le résultat a un sens pour les humains, on l'affiche
                if solde_impossible(solde_initial, nb_des) { continue; }
                let esperance_gain = esperance_avec_relances - solde_initial as Flottant;
                print!("- Solde initial {}: Esperance >= {} (Gain moyen >= {:+}, ",
                       solde_initial, esperance_avec_relances, esperance_gain);
                if delta > 0. {
                    print!("Delta = {:+}", delta);
                } else {
                    print!("STABLE");
                }
                println!(")");
            }

            println!();
        }

        if !continuer { break; }
    }

    // TODO: Faire des combats de robots
    // TODO: Etudier l'atterissage
}

// Calcule de l'espérance de gain pour une stratégie où on relance les dés
// jusqu'à "profondeur" fois en partant d'un certain solde initial et d'un
// certain nombre de dés. Pour une profondeur N donnée, le code suppose que
// toutes les profondeurs 0..N précédentes ont déjà été sondées.
fn iterer_esperance(profondeur: usize, stats_jets: &[StatsJet], stats_jet_actuel: &StatsJet, solde_initial: Valeur) -> Flottant {
    // Le but est de déterminer une espérance de gain pour un certain lancer
    let mut esperance_lancer = 0.;

    // On passe en revue tous les résultats de lancers gagnants
    for stats_choix in stats_jet_actuel.stats_choix.iter() {
        // Est-ce que, par chance, j'ai déjà étudié ce cas à une itération
        // précédente du calcul en cours?
        if let Some(esperance_max) = stats_choix.esperance_max.borrow().get(&(profondeur, solde_initial)) {
            esperance_lancer += esperance_max * stats_choix.proba;
            continue;
        }

        // Après avoir obtenu des combinaisons, il faut choisir la stratégie
        // optimale entre empocher l'une des combinaisons disponibles (la plus
        // chère de préférence) et relancer les dés N fois. Les cas où on
        // relance les dés <N fois ont déjà été traités, c'est donc seulement
        // le cas où on relance exactement N fois qui nous intéresse.
        let mut esperance_max = if profondeur == 0 {
            // A zéro relances, le cas initial c'est celui où on ne lance pas
            solde_initial as Flottant
        } else {
            // A N>0 relances, c'est celui où on lance N-1 fois
            stats_choix.esperance_max.borrow()[&(profondeur - 1, solde_initial)]
        };

        // On étudie ensuite toutes les possibilités de relance une à une, dans
        // une stratégie où on relance toujours jusqu'à la profondeur N (les
        // profondeurs <N ont été déjà examinées et intégrées à esperance_max)
        for poss in stats_choix.choix.iter() {
            // Est-ce qu'on a atteint le nombre de relances désiré?
            let esperance_relance = if profondeur == 0 {
                // Si oui, on prend les gains systématiquement.
                (solde_initial + poss.valeur) as Flottant
            } else {
                // Sinon, ajoute le solde à notre gain et relance à notre
                // nouveau nombre de dés, en décrémentant le budget relance?
                iterer_esperance(profondeur - 1,
                                 stats_jets,
                                 &stats_jets[poss.nb_des_relance - 1],
                                 solde_initial + poss.valeur)
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
                       .insert((profondeur, solde_initial), esperance_max),
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