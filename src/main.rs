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

// Soldes pour lesquels on estime les espérances de gain
const NB_SOLDES : usize = 16;
const SOLDES : [Valeur; NB_SOLDES] = [0, 50, 100, 150, 200, 250, 300, 350, 400,
                                      900, 950, 1000, 2700, 2800, 2900, 10000];

// On va compter les probas sur 64-bit au cas où, on diminuera si besoin
type Flottant = f32;

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face, si on tire des combinaisons
    stats_choix: Vec<StatsChoix>,

    // Espérance si on lance ce nombre de dés et s'arrête là
    esperance_sans_relance: [Flottant; NB_SOLDES],

    // Espérance à la dernière profondeur de relance calculée
    esperance_actuelle: Cell<[Flottant; NB_SOLDES]>,

    // Probabilité de tirer une combinaison gagnante
    proba_gain: Flottant,
}

// L'un dex choix face auxquels un jet de dés peut nous placer
struct StatsChoix {
    // Combinaisons entre lesquels il faut choisir
    choix: Vec<Possibilite>,

    // Probabilité qu'on a de faire face à ce choix
    proba: Flottant,

    // Valeur de la combinaison la plus chère qu'on puisse choisir
    valeur_max: Valeur,

    // Pour un solde initial donné, espérance de la possibilité la plus
    // profitable aux profondeurs de relance considérées précédemment
    esperance_max: HashMap<Valeur, Flottant>,

    // Tampon permettant de construire esperance_max pour la profondeur de
    // relance suivante sans perturber le calcul de la profondeur en cours
    future_esperance_max: RefCell<HashMap<Valeur, Flottant>>,
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

                    // On annote chaque choix avec la valeur max de combinaison
                    let valeur_max = choix.iter().map(|p| p.valeur).max().unwrap();

                    // On transforme notre comptage en probabilité
                    StatsChoix {
                        choix,
                        proba,
                        valeur_max,
                        esperance_max: HashMap::new(),
                        future_esperance_max: RefCell::new(HashMap::new()),
                    }
                }).collect::<Vec<StatsChoix>>();

        // La probabilité de gagner est plus utile que la probabilité de perdre,
        // elle dit quelle proportion du solde on garde en moyenne en relançant
        let proba_gain = 1. - proba_perte;
        println!("Probabilité de gagner à {} dés: {}", nb_des, proba_gain);

        // On peut aussi calculer combien on gagne en moyenne si on lance N dés
        // et s'arrête là. C'est une borne inférieure de ce qu'on peut gagner
        // dans une stratégie optimale avec relance, en partant d'un solde nul.
        let esperance_jet : Flottant =
            stats_choix.iter()
                .map(|s| s.valeur_max as Flottant * s.proba)
                .sum();

        // On intègre ensuite la présence d'un solde préalable en prenant en
        // compte la probabilité de perdre ce solde.
        let mut esperance_sans_relance = [0.; NB_SOLDES];
        println!("Espérance sans relancer:");
        for (idx_solde, &solde_initial) in SOLDES.iter().enumerate() {
            let valeur_amortie = solde_initial as Flottant * proba_gain;
            let esperance = valeur_amortie + esperance_jet;
            let esperance_gain = esperance - solde_initial as Flottant;
            println!("- Solde initial {}: Espérance >= {} (Gain moyen >= {:+})",
                     solde_initial, esperance, esperance_gain);
            esperance_sans_relance[idx_solde] = esperance;
        }

        // Nous gardons de côté ces calculs, on a besoin de les avoir effectués
        // pour tous les nombres de dés avant d'aller plus loin.
        stats_jets.push(StatsJet {
            stats_choix,
            esperance_sans_relance,
            esperance_actuelle: Cell::new(esperance_sans_relance),
            proba_gain,
        });

        println!();
    }

    // Maintenant, on explore le graphe de décision avec un nombre croissant
    // de relances autorisées (...et un temps de calcul qui explose)
    for profondeur in 1..=5 {
        println!("=== JETS AVEC <={} RELANCES ===\n", profondeur);

        // Maintenant, on reprend le calcul en autorisant une relance
        for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
            println!("Espérances à {} dés:", idx_nb_des + 1);

            // Comme précédemment, on considère différents soldes de départ
            for (idx_solde, &solde_initial) in SOLDES.iter().enumerate() {
                // On calcule l'espérance à profondeur N récursivement (cf ci-dessous)
                let esperance_avec_relances = iterer_esperance(profondeur, &stats_jets[..], stats, solde_initial, idx_solde);

                // On affiche le résultat
                let esperance_gain = esperance_avec_relances - solde_initial as Flottant;
                print!("- Solde initial {}: Esperance >= {} (Gain moyen >= {:+}",
                       solde_initial, esperance_avec_relances, esperance_gain);

                // On vérifie que les espérances vont bien croissantes
                let mut esperances = stats.esperance_actuelle.get();
                assert!(esperance_avec_relances >= esperances[idx_solde]);
                print!(", Delta = {:+}", esperance_avec_relances - esperances[idx_solde]);
                if esperance_avec_relances == esperances[idx_solde] { print!(" => STABLE"); }
                esperances[idx_solde] = esperance_avec_relances;
                stats.esperance_actuelle.set(esperances);
                println!(")");
            }

            println!();
        }

        // Après chaque remplissage, on propage future_esperance_max vers esperance_max
        // afin que le calcul pour la profondeur suivante prenne bien en compte les
        // résultats de la profondeur actuelle.
        for stats in stats_jets.iter_mut() {
            for stats_choix in stats.stats_choix.iter_mut() {
                stats_choix.esperance_max = stats_choix.future_esperance_max.borrow().clone();
            }
        }
    }

    // TODO: Faire des combats de robots
    // TODO: Etudier l'atterissage
}

// Ayant préalablement calculé l'espérance de gain pour une stratégie optimale
// où on relance jusqu'à N-1 fois, on veut la calculer pour une stratégie
// optimale où on relance jusqu'à N fois, en incorporants aux résultats
// précédents les relances de profondeur N.
#[inline(always)]
fn iterer_esperance(profondeur: usize, stats_jets: &[StatsJet], stats: &StatsJet, solde_initial: Valeur, idx_solde: usize) -> Flottant {
    // Le but est de déterminer une espérance de gain en cas de relance
    let mut esperance_avec_relances = 0.;

    // On passe en revue tous les résultats de lancers gagnants
    for stats_choix in stats.stats_choix.iter() {
        // Après avoir obtenu des combinaisons, nous devons choisir si il faut
        // prendre celle de valeur la plus élevée et s'arrêter, ou relancer. Si
        // on relance, il faut aussi décider quelle combinaison on prend.
        //
        // On note la valeur du choix optimal pour les profondeurs de relance
        // considérées jusqu'à présent, en l'initialisant avec la valeur de la
        // combinaison la plus chère (valeur du cas sans relance)
        //
        let mut esperance_max =
            stats_choix.esperance_max
                .get(&solde_initial)
                .cloned()
                .unwrap_or((solde_initial + stats_choix.valeur_max) as Flottant);

        // On étudie ensuite toutes les possibilités de relance une à une, dans
        // une stratégie où on relance toujours jusqu'à la profondeur N (les
        // profondeurs <N ont été déjà examinées et intégrées à esperance_max)
        for poss in stats_choix.choix.iter() {
            // Voyons les stats pour le nombre de dés qu'on relancerait
            let stats_des_relance = &stats_jets[poss.nb_des_relance-1];

            // Est-ce qu'on est à la profondeur de relance désirée?
            let esperance_relance = match profondeur {
                1 => {
                    // Si oui, on ne relancera plus, on peut donc calculer
                    // l'espérance de gain en combinant l'espérance de perte des
                    // sommes que nous avons accumulées jusqu'ici avec
                    // l'espérance de gain sans relance des dés qu'on jette
                    let valeur_amortie = poss.valeur as Flottant * stats_des_relance.proba_gain;
                    valeur_amortie + stats_des_relance.esperance_sans_relance[idx_solde]
                }
                i if i > 1 => {
                    // Sinon, on relance toujours: on poursuit le calcul
                    // récursivement en considérant chaque possibilité de
                    // relance à partir du jet en cours, pour calculer notre
                    // espérance de gain à partir de ce jet.
                    iterer_esperance(profondeur-1, stats_jets, stats_des_relance, solde_initial + poss.valeur, idx_solde)
                }
                _ => unreachable!()
            };

            // Si relancer après avoir pris cette combinaison rapporte plus que
            // toutes les alternatives considérées jusqu'à présent, cela devient
            // notre nouvelle stratégie de prédilection.
            esperance_max = esperance_max.max(esperance_relance);
        }

        // En intégrant les stratégies optimales (à cette profondeur de relance)
        // sur tous les lancers de dés possibles, on en déduit l'espérance de
        // gain pour une stratégie optimale à <= N relances.
        esperance_avec_relances += esperance_max * stats_choix.proba;

        // On garde de côté l'espérance max, qui resservira lorsqu'on calculera
        // l'espérance des relances plus profondes.
        stats_choix.future_esperance_max.borrow_mut().entry(solde_initial)
            .and_modify(|e| *e = e.max(esperance_max))
            .or_insert(esperance_max);
    }

    esperance_avec_relances

}