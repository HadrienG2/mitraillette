mod choix;
mod combinaison;

use crate::combinaison::{Combinaison, Valeur};

use std::{
    cell::Cell,
    fmt::{self, Debug},
};


// Nombre de dés maximum qu'on peut lancer
const NB_DES_TOT : usize = 6;

// Nombre de faces par dé
const NB_FACES : usize = 6;

// Soldes pour lesquels on estime les espérances de gain
const NB_SOLDES : usize = 28;
const SOLDES : [Valeur; NB_SOLDES] = [0, 50, 100, 150, 200, 250, 300, 400, 500,
                                      600, 700, 800, 900, 1000, 1100, 1200,
                                      1300, 1400, 1500, 1600, 1700, 1800, 1900,
                                      2000, 2300, 2600, 2900, 3200];

// On va compter les probas sur 64-bit au cas où, on diminuera si besoin
type Flottant = f32;

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face, si on tire des combinaisons
    stats_choix: Vec<StatsChoix>,

    // Espérance de gain sans relancer, pour ce nombre de dés et chaque solde
    // initial considéré
    esperance_gain_sans_relancer: [Flottant; NB_SOLDES],

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
}

// L'une des possibilités entre lesquelles il faut choisir
struct Possibilite {
    // Combinaison qu'on décide ou non de choisir
    comb: Combinaison,

    // Valeur de cette combinaison
    valeur: Valeur,

    // Nombre de dés avec lequel on peut relancer ensuite
    nb_des_relance: usize,

    // Borne inférieure de l'espérance de gain si on relance une fois, pour
    // chaque solde initial considéré
    min_esperance_relance_simple: Cell<[Flottant; NB_SOLDES]>,
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
                                min_esperance_relance_simple: Cell::new([0.; NB_SOLDES]),
                            }
                        }).collect::<Vec<_>>();

                    // On annote chaque choix avec la valeur max de combinaison
                    let valeur_max = choix.iter().map(|p| p.valeur).max().unwrap();

                    // On transforme notre comptage en probabilité
                    StatsChoix {
                        choix,
                        proba,
                        valeur_max,
                    }
                }).collect::<Vec<StatsChoix>>();

        // La probabilité de gagner est plus utile que la probabilité de perdre,
        // car elle dit quelle proportion de son solde on garde en moyenne quand
        // on relance les dés.
        let proba_gain = 1. - proba_perte;
        println!("Probabilité de gagner à {} dés: {}", nb_des, proba_gain);

        // On peut aussi calculer l'espérance de gain sans relance (on lance les
        // dés, on prend la combinaison la plus élevée, et on s'arrête là).
        //
        // C'est une borne inférieure de l'espérance de gain réelle, puisqu'on
        // ne relancera pour gagner plus que si la relance rapporte en moyenne
        // plus que le gain maximal obtenu en s'arrêtant là.
        //
        let esperance_jet : Flottant =
            stats_choix.iter()
                .map(|s| s.valeur_max as Flottant * s.proba)
                .sum();
        let mut esperance_gain_sans_relancer = [0.; NB_SOLDES];

        // On peut en déduire, moyennant un certain solde initial, combien on
        // peut espérer gagner en relançant et en s'arrêtant là.
        println!("Espérance sans relancer:");
        for (idx_solde, &solde_initial) in SOLDES.iter().enumerate() {
            let valeur_amortie = solde_initial as Flottant * proba_gain;
            let min_esperance = valeur_amortie + esperance_jet;
            println!("- Solde initial {}: {}", solde_initial, min_esperance);
            esperance_gain_sans_relancer[idx_solde] = min_esperance;
        }

        // Nous gardons de côté ces calculs, on a besoin de les avoir effectués
        // pour tous les nombres de dés avant d'aller plus loin.
        stats_jets.push(StatsJet {
            stats_choix,
            esperance_gain_sans_relancer,
            proba_gain,
        });

        println!();
    }

    println!("=== PRISE EN COMPTE DE RELANCES UNIQUES ===\n");

    // On affine notre borne inférieure de l'espérance de gain en considérant
    // une seule relance par jet
    for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
        println!("Espérances de gain à {} dés avec relance unique:", idx_nb_des + 1);

        for (idx_solde, &solde_initial) in SOLDES.iter().enumerate() {
            let mut esperance_gain_relance_unique = 0.;

            // On passe en revue tous les choix auxquels on peut faire face
            for stat_choix in stats.stats_choix.iter() {
                // Pour chaque choix, on calcule une borne inférieure de
                // l'espérance de gain en cas de relance, et voir si elle est
                // supérieure à ce qu'on est sûr de gagner en s'arrêtant là
                let mut max_min_esperance_relance: Flottant = (solde_initial + stat_choix.valeur_max) as Flottant;

                // Pour cela, on étudie les relances pour chaque combinaison...
                for poss in stat_choix.choix.iter() {
                    // Voyons les stats pour le nombre de dés qu'on relancerait
                    let stats_nouv_des = &stats_jets[poss.nb_des_relance-1];

                    // La probabilité de jet gagnant nous dit quelle proportion
                    // de la valeur on garde en moyenne en relançant
                    let valeur_amortie = poss.valeur as Flottant * stats_nouv_des.proba_gain;

                    // On y ajoute notre borne inférieure de ce qu'on espère
                    // gagner en relançant les dés restants une seule fois,
                    // calcul qu'on a déjà effectué précédemment.
                    let min_esperance_relance = valeur_amortie + stats_nouv_des.esperance_gain_sans_relancer[idx_solde];

                    // On garde cette quantité de côté, elle sera utile quand on
                    // s'autorisera à relancer deux fois.
                    let mut nouv_esp_relance = poss.min_esperance_relance_simple.get();
                    nouv_esp_relance[idx_solde] = min_esperance_relance;
                    poss.min_esperance_relance_simple.set(nouv_esp_relance);

                    // En calculant le maximum de ces bornes inférieures pour toutes
                    // les relances possibles, on en tire une borne inférieure de
                    // l'espérance de gain en cas de relance optimale.
                    max_min_esperance_relance = max_min_esperance_relance.max(min_esperance_relance);
                }

                // Avec le maximum du gain assuré (pour ce choix) et de notre
                // borne inférieure de l'espérance de gain en cas de relance
                // simple, on a une borne inférieure de l'espérance de gain pour
                // une de stratégie de relance ou non-relance optimale.
                //
                // FIXME: Il faudrait garder max_min_esperance_relance en cache
                //        pour le prochain calcul, où on y intégrera les
                //        possibilités en relançant deux fois.
                //
                esperance_gain_relance_unique += max_min_esperance_relance * stat_choix.proba;
            }

            println!("- Solde initial {}: {}", solde_initial, esperance_gain_relance_unique);
            assert!(esperance_gain_relance_unique > stats.esperance_gain_sans_relancer[idx_solde]);
        }
        // TODO: Calculer pour les autres soldes

        println!();
    }

    println!("=== PRISE EN COMPTE DE RELANCES DOUBLES ===\n");

    // FIXME: Forte répétition avec ci-dessus, à dédupliquer
    for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
        println!("Espérances de gain à {} dés avec relance double:", idx_nb_des + 1);

        for (idx_solde, &solde_initial) in SOLDES.iter().enumerate() {
            let mut esperance_gain_relance_double = 0.;

            for stat_choix in stats.stats_choix.iter() {
                // FIXME: Une meilleure valeur initiale serait le max_min_esperance_relance précédent
                let mut max_min_esperance_relance: Flottant = (solde_initial + stat_choix.valeur_max) as Flottant;;

                for poss in stat_choix.choix.iter() {
                    // Relance simple: on relance, et on prend ce qui sort
                    // TODO: Copie du calcul précédent, peut être éliminée en partant du bon max_min_esperance_relance
                    let stats_nouv_des = &stats_jets[poss.nb_des_relance-1];
                    let valeur_amortie = poss.valeur as Flottant * stats_nouv_des.proba_gain;
                    max_min_esperance_relance = max_min_esperance_relance.max(poss.min_esperance_relance_simple.get()[idx_solde]);

                    // Relance double: on relance, et on relance encore
                    // TODO: Exprimable en fonction des calculs précédents?
                    let mut esperance_gain_relance_double_2 = 0.;
                    for stats_choix_2 in stats_nouv_des.stats_choix.iter() {
                        let mut max_min_esperance_relance_2: Flottant = (solde_initial + stat_choix.valeur_max + stats_choix_2.valeur_max) as Flottant;
                        for poss_2 in stats_choix_2.choix.iter() {
                            let stats_nouv_des_2 = &stats_jets[poss_2.nb_des_relance-1];
                            let valeur_amortie_2 = valeur_amortie * stats_nouv_des_2.proba_gain;
                            let min_esperance_relance_2 = poss_2.min_esperance_relance_simple.get()[idx_solde] + valeur_amortie_2;
                            // Le résultat de ce max sera donc peut-être différent, donc à partir de là ça change
                            max_min_esperance_relance_2 = max_min_esperance_relance_2.max(min_esperance_relance_2);
                        }
                        esperance_gain_relance_double_2 += max_min_esperance_relance_2 * stats_choix_2.proba;
                    }
                    max_min_esperance_relance = max_min_esperance_relance.max(esperance_gain_relance_double_2);
                }

                esperance_gain_relance_double += max_min_esperance_relance * stat_choix.proba;
            }

            println!("- Solde initial {}: {}", solde_initial, esperance_gain_relance_double);
             // FIXME: Stocker espérance de gain à relance unique et comparer
             // FIXME: Si ça se stabilise, le calcul semble converger
            assert!(esperance_gain_relance_double > stats.esperance_gain_sans_relancer[idx_solde]);
        }

        println!();
    }

    // TODO: Prendre en compte les relances triples, etc...
    // TODO: Faire des combats de robots
    // TODO: Etudier l'atterissage
}
