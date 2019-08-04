mod choix;
mod combinaison;

use crate::combinaison::{Combinaison, Valeur};

use std::{
    cell::Cell,
    fmt::{self, Debug},
    collections::HashMap,
};


// Nombre de dés maximum qu'on peut lancer
const NB_DES_TOT : usize = 6;

// Nombre de faces par dé
const NB_FACES : usize = 6;

// Histogramme d'un jet de dé par face (nb de dés tombé sur chaque face)
type HistogrammeFaces = [usize; NB_FACES];

// On va compter les probas sur 64-bit au cas où, on diminuera si besoin
type Flottant = f32;

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face, si on tire des combinaisons
    stats_choix: Vec<StatsChoix>,

    // Espérance de gain sans relancer pour ce nombre de dés
    esperance_gain_sans_relancer: Flottant,

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

    // Borne inférieure de l'espérance de gain si on relance une fois
    min_esperance_relance_simple: Cell<Flottant>,
}

impl Debug for Possibilite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?} ({}pt, rel. {}d)",
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
        let nb_comb = NB_FACES.pow(nb_des as u32);
        println!("Nombre de lancers possibles à {} dés: {}", nb_des, nb_comb);

        // On énumère tous les lancers possibles pour ce nombre de dés
        let mut comptage_choix = HashMap::new();
        for num_comb in 0..nb_comb {
            // On énumère les faces en traitant la combinaison comme un nombre
            // en base NB_FACES (note: la face 1 est numérotée 0), et on calcule
            // l'histogramme du nombre de dés étant tombé sur chaque face.
            let mut reste = num_comb;
            let mut histo = [0; NB_FACES];
            for _ in 0..nb_des {
                let idx_face = reste % NB_FACES;
                histo[idx_face] += 1;
                reste /= NB_FACES;
            }

            // On déduit de cet histogramme les combinaisons entre lesquelles
            // on peut raisonnablement choisir...
            let choix = choix::enumerer_combinaisons(histo);

            // ...et on en compte les occurences, dont on déduira la probabilité
            let compte = comptage_choix.entry(choix.clone()).or_insert(0);
            *compte += 1;
        }

        // Nous en tirons une table des choix auxquels on peut faire face, et du
        // nombre de tirages où ils se présentent. On transforme les comptes en
        // probas, et on complète la table des choix avec diverses annotations.
        let norme = 1. / (nb_comb as Flottant);
        let mut proba_perte = 0.;
        let stats_choix =
            comptage_choix.into_iter()
                .filter(|(choix, compte)| {
                    // On met de côté le cas perdant, qui est spécial (on ne
                    // peut pas choisir de continuer ou d'arrêter, on perd
                    // toujours), on ne garde que sa probabilité.
                    if choix == &[] {
                        proba_perte = *compte as Flottant * norme;
                        false
                    } else {
                        true
                    }
                })
                .map(|(choix, compte)| {
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
                                min_esperance_relance_simple: Cell::new(0.),
                            }
                        }).collect::<Vec<_>>();

                    // On annote chaque choix avec la valeur de sa combinaison
                    // la plus chère
                    let valeur_max = choix.iter()
                                          .map(|poss| poss.valeur)
                                          .max()
                                          .unwrap_or(0);

                    // On transforme notre comptage en probabilité
                    StatsChoix {
                        choix,
                        proba: (compte as Flottant) * norme,
                        valeur_max,
                    }
                }).collect::<Vec<StatsChoix>>();

        // La probabilité de gagner est plus utile que la probabilité de perdre,
        // car elle dit quelle proportion de son solde on garde en moyenne quand
        // on relance les dés.
        let proba_gain = 1. - proba_perte;
        println!("Probabilité de gagner: {}", proba_gain);

        // On peut aussi calculer l'espérance de gain sans relance (on lance les
        // dés, on prend la combinaison la plus élevée, et on s'arrête là).
        //
        // C'est une borne inférieure de l'espérance de gain réelle, puisqu'on
        // ne relancera pour gagner plus que si la relance rapporte en moyenne
        // plus que le gain maximal obtenu en s'arrêtant là.
        //
        let esperance_gain_sans_relancer : Flottant =
            stats_choix.iter()
                .map(|s| s.valeur_max as Flottant * s.proba)
                .sum();
        println!("Espérance sans relancer: {}", esperance_gain_sans_relancer);

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
        let mut esperance_gain_relance_unique = 0.;

        // On passe en revue tous les choix auxquels on peut faire face
        for stat_choix in stats.stats_choix.iter() {
            // Pour chaque choix, on calcule une borne inférieure de
            // l'espérance de gain en cas de relance, sans solde préalable.
            let mut max_min_esperance_relance: Flottant = 0.;

            // Pour cela, on étudie les relances pour chaque combinaison...
            for poss in stat_choix.choix.iter() {
                // Voyons les stats pour le nombre de dés qu'on relancerait
                let stats_nouv_des = &stats_jets[poss.nb_des_relance-1];

                // La probabilité de jet gagnant nous dit quelle proportion
                // de la valeur on garde en moyenne en relançant
                let valeur_amortie = poss.valeur as Flottant * stats_nouv_des.proba_gain;

                // On y ajoute notre borne inférieure de ce qu'on espère
                // gagner en relançant les dés restants une seule fois.
                let min_esperance_relance = valeur_amortie + stats_nouv_des.esperance_gain_sans_relancer;

                // On garde cette quantité de côté, elle sera utile quand on
                // s'autorisera à relancer deux fois.
                poss.min_esperance_relance_simple.set(min_esperance_relance);

                // En calculant le maximum de ces bornes inférieures pour toutes
                // les relances possibles, on en tire une borne inférieure de
                // l'espérance de gain en cas de relance optimale.
                max_min_esperance_relance = max_min_esperance_relance.max(min_esperance_relance);
            }

            // A ce stade, on sait ce qu'on gagne si on s'arrête là, et on a une
            // borne inférieure de ce qu'on gagne en cas de relance optimale. En
            // choisissant le plus avantageux des deux, on a une borne
            // inférieure de ce qu'on gagne en choisissant ou non de relancer
            // dans une stratégie de relance optimale.
            let borne_inf_gain = max_min_esperance_relance.max(stat_choix.valeur_max as Flottant);
            esperance_gain_relance_unique += borne_inf_gain * stat_choix.proba;
        }

        println!("Espérance de gain à {} dés avec relance unique: {}",
                 idx_nb_des + 1, esperance_gain_relance_unique);
        // FIXME: Ce calcul n'est pas correct, car il sous-estime la probabilité
        //        de perdre le solde initial, en ne considérant que la
        //        possibilité de perdre lors du premier lancer (et pas de la
        //        relance éventuelle).
        //
        //        Pour effectuer ce calcul, il me faudrait au moins une borne de
        //        la probabilité de perdre le gain dans une stratégie de relance
        //        optimale... ce qui dépend probablement du solde initial.
        //
        /* println!("Lancer clairement pertinent si solde préalable < {}",
                 esperance_gain_relance_unique / (1. - stats.proba_gain));
        println!(); */
    }

    println!("\n=== PRISE EN COMPTE DE RELANCES DOUBLES ===\n");

    // TODO: Prendre en compte les relances doubles, triples...

    // WIP, copié-collé de ci-dessus
    for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
        let mut esperance_gain_relance_double = 0.;

        for stat_choix in stats.stats_choix.iter() {
            let mut max_min_esperance_relance: Flottant = 0.;

            for poss in stat_choix.choix.iter() {
                // Relance simple: on relance, et on prend ce qui sort
                // TODO: Copie du calcul précédent, peut être éliminée en mettant en cache
                let stats_nouv_des = &stats_jets[poss.nb_des_relance-1];
                let valeur_amortie = poss.valeur as Flottant * stats_nouv_des.proba_gain;
                let min_esperance_relance_simple = poss.min_esperance_relance_simple.get();
                max_min_esperance_relance = max_min_esperance_relance.max(min_esperance_relance_simple);

                // Relance double: on relance, et on relance encore
                // TODO: Exprimable en fonction des calculs précédents?
                let mut esperance_gain_relance_double_2 = 0.;
                for stats_choix_2 in stats_nouv_des.stats_choix.iter() {
                    let mut max_min_esperance_relance_2: Flottant = 0.;
                    for poss_2 in stats_choix_2.choix.iter() {
                        let stats_nouv_des_2 = &stats_jets[poss_2.nb_des_relance-1];
                        let valeur_amortie_2 = valeur_amortie * stats_nouv_des_2.proba_gain;
                        let min_esperance_relance_2 = poss_2.min_esperance_relance_simple.get() + valeur_amortie_2;
                        // Le résultat de ce max sera donc peut-être différent, donc à partir de là ça change
                        max_min_esperance_relance_2 = max_min_esperance_relance_2.max(min_esperance_relance_2);
                    }
                    // ...et donc là ça change aussi
                    let borne_inf_gain_2 = max_min_esperance_relance_2.max(stat_choix.valeur_max as Flottant);
                    esperance_gain_relance_double_2 += borne_inf_gain_2 * stats_choix_2.proba;
                }
                max_min_esperance_relance = max_min_esperance_relance.max(esperance_gain_relance_double_2);
            }

            let borne_inf_gain = max_min_esperance_relance.max(stat_choix.valeur_max as Flottant);
            esperance_gain_relance_double += borne_inf_gain * stat_choix.proba;
        }

        println!("Espérance de gain à {} dés avec relance double: {}",
                 idx_nb_des + 1, esperance_gain_relance_double);
        // FIXME: Ce calcul n'est pas correct, car il sous-estime la probabilité
        //        de perdre le solde initial, en ne considérant que la
        //        possibilité de perdre lors du premier lancer (et pas de la
        //        relance éventuelle).
        //
        //        Pour effectuer ce calcul, il me faudrait au moins une borne de
        //        la probabilité de perdre le gain dans une stratégie de relance
        //        optimale... ce qui dépend probablement du solde initial.
        //
        /* println!("Lancer clairement pertinent si solde préalable < {}",
                 esperance_gain_relance_doubme / (1. - stats.proba_gain));
        println!(); */
    }

    // TODO: Faire l'étude de fonction espérance à différents soldes, ne pas
    //       se cantonner au solde nul (qui n'est valide qu'à 6 dés)
}
