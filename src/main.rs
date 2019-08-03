mod choix;
mod combinaison;

use crate::{
    choix::Choix,
    combinaison::Valeur,
};

use std::{
    cell::Cell,
    cmp::Ordering,
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

// Si on l'ensemble des jets de dés possibles à N dés, on se retrouve avec une
// table des choix face auxquels on peut se retrouver en lançant les dés, et
// des probabilités associées.
struct StatsChoix {
    // Choix de combinaisons auquel on fait face
    choix: Choix,

    // Probabilité qu'on a de faire face à ce choix
    proba: Flottant,

    // Valeur de la combinaison la plus chère
    valeur_max: Valeur,
}

// Ce qu'on sait sur les jets d'un certain nombre de dés
struct StatsJet {
    stats_choix: Vec<StatsChoix>,
    min_esperance_gain: Cell<Flottant>,
    proba_gain: Flottant,
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
        let mut comptage_choix: HashMap<Choix, u16> = HashMap::new();
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
            let choix = choix::enumerer_choix(histo);

            // ...et on en compte les occurences, dont on déduira la probabilité
            let compte = comptage_choix.entry(choix.clone()).or_insert(0);
            *compte += 1;
        }

        // Nous en tirons une table des choix face auxquels on peut se
        // retrouver, avec la probabilité de chacun et la valeur de la
        // combinaison la plus chère.
        let norme = 1. / (nb_comb as Flottant);
        let mut stats_choix =
            comptage_choix.into_iter()
                .map(|(choix, compte)| {
                    let valeur_max = choix.iter()
                                          .map(|comb| comb.valeur())
                                          .max()
                                          .unwrap_or(0);
                    StatsChoix {
                        choix,
                        proba: (compte as Flottant) * norme,
                        valeur_max,
                    }
                }).collect::<Vec<StatsChoix>>();

        // On trie la table, ce qui la rend plus lisible pour l'affichage, et
        // place le cas sans combinaison au début. On récupère la probabilité de
        // celui-ci, puis on l'écarte puisqu'il est spécial (c'est le seul
        // cas où on ne peut pas prendre, il n'y a donc pas de choix).
        stats_choix.sort_unstable_by(|s1, s2| s1.choix.cmp(&s2.choix));
        let proba_gain = 1. - stats_choix.remove(0).proba;
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

        // Pour terminer, on énumère les combinaisons possibles
        println!("Choix auxquels on peut faire face:");
        for s in stats_choix.iter() {
            println!("- {:?} (Probabilité: {}, Valeur max: {})",
                     s.choix, s.proba, s.valeur_max);
        }

        // Nous gardons de côté ces calculs, on a besoin de les avoir effectués
        // pour tous les nombres de dés avant d'aller plus loin.
        stats_jets.push(StatsJet {
            stats_choix,
            min_esperance_gain: Cell::new(esperance_gain_sans_relancer),
            proba_gain,
        });

        println!();
    }

    println!("=== PRISE EN COMPTE DES RELANCES SIMPLES ===\n");

    // Maintenant, on peut affiner notre borne inférieure de l'espérance de gain
    // par récurence. L'idée générale est de considérer une stratégie où on
    // relance à chaque fois que notre borne inférieure de l'espérance de gain
    // dit que c'est favorable, et de mettre à jour notre espérance de gain
    // en fonction de ce nouveau résultat. Cela augmentera l'espérance de gain,
    // ce qui peut affecter la stratégie ci-dessus, donc il faut itérer.
    loop {
        let mut continuer = false;

        // Pour chaque nombre de dés...
        for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
            let nb_des = idx_nb_des + 1;

            // ...on veut affiner notre borne inférieure de l'espérance de gain
            let ancienne_esperance_min = stats.min_esperance_gain.get();
            let mut nouvelle_esperance_min = 0.;

            // On passe en revue tous les choix auxquels on peut faire face
            // FIXME: Ne rien afficher pendant la récurence, seulement à la fin.
            println!("Cas à {} dés", nb_des);
            for s in stats.stats_choix.iter() {
                println!("- Choix: {:?} (Proba: {}, Valeur max: {})",
                         s.choix, s.proba, s.valeur_max);

                // Pour chaque choix, on calcule une borne inférieure à
                // l'espérance de gain en cas de relance sans solde préalable.
                let mut esperance_min_sans_solde: Flottant = 0.;

                // Pour cela, on énumère les combinaisons...
                for comb in s.choix.iter() {
                    println!("  * Combinaison: {:?}", comb);
                    println!("    o Valeur sans relance: Solde + {}", comb.valeur());
                    let des_restants = nb_des - comb.nb_des();
                    let nouv_nb_des = if des_restants == 0 { 6 } else { des_restants };
                    let stats_nouv_des = &stats_jets[nouv_nb_des-1];
                    let esperance_min = stats_nouv_des.min_esperance_gain.get();
                    println!("    o Nouveau nombre de dés: {} (Probabilité de gain: {}, Espérance >= {})",
                             nouv_nb_des, stats_nouv_des.proba_gain, esperance_min);
                    let valeur_amortie = comb.valeur() as Flottant * stats_nouv_des.proba_gain;
                    println!("    o Espérance avec relance: Solde * {} + {} + Espérance({} dés | Solde=0)",
                             stats_nouv_des.proba_gain, valeur_amortie, nouv_nb_des);
                    let borne_inf_sans_solde = valeur_amortie + esperance_min;
                    println!("    o Dans le cas où Solde = 0, {} + Espérance({} dés | Solde=0) >= {}",
                             valeur_amortie, nouv_nb_des, borne_inf_sans_solde);
                    esperance_min_sans_solde = esperance_min_sans_solde.max(borne_inf_sans_solde);
                }

                // On en déduit si il faut clairement relancer, et on intègre le
                // résultat maximal (valeur max si on s'arrête ou borne
                // inférieure de l'espérance de gain en cas de relance)
                println!("  * En conclusion, dans le cas où Solde = 0...");
                println!("    o Espérance avec relance >= {}", esperance_min_sans_solde);
                if esperance_min_sans_solde > s.valeur_max as Flottant {
                    println!("    o Il faut toujours relancer!");
                    nouvelle_esperance_min += esperance_min_sans_solde * s.proba;
                } else {
                    println!("    o On ne peut pas conclure pour l'instant...");
                    nouvelle_esperance_min += s.valeur_max as Flottant * s.proba;
                }
            }

            // Si l'espérance de gain a augmenté dans cette itération, on la met
            // à jour et on note qu'il faut continuer d'itérer (puisque cela a
            // une influence sur les résultats pour d'autres nombres de faces).
            match nouvelle_esperance_min.partial_cmp(&ancienne_esperance_min) {
                Some(Ordering::Greater) => {
                    println!("- L'espérance minimale augmente de {} à {}",
                             ancienne_esperance_min, nouvelle_esperance_min);
                    stats.min_esperance_gain.set(nouvelle_esperance_min);
                    continuer = true;
                }
                Some(Ordering::Equal) => {
                    println!("- L'espérance minimale reste à {}",
                             ancienne_esperance_min);
                }
                _ => unreachable!()
            }

            println!();
        }

        // On sépare les résultats de différentes itérations
        if continuer {
            println!("------\n");
        } else {
            break;
        }
    }

    // TODO: Que faire ensuite?
}
