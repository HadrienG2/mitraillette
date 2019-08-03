mod choix;
mod combinaison;

use crate::choix::Choix;
use std::collections::HashMap;


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
type ChoixEtProba = (Choix, Flottant);

// Ce qu'on sait sur les jets d'un certain nombre de dés
struct StatsJet {
    choix_et_probas: Vec<ChoixEtProba>,
    esperance_sans_relance: Flottant,
    proba_perte: Flottant,
}


fn main() {
    println!("\n=== ORDRE 0: JETS ISOLES ===\n");

    // Tout d'abord, pour chaque nombre de dés, on calcule face à quels choix
    // on peut se retrouver, et avec quelle probabilité
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

            // On énumère les combinaisons qu'on peut prendre
            let choix = choix::enumerer_choix(histo);

            // On compte la fréquence de chaque situation...
            let compte = comptage_choix.entry(choix.clone()).or_insert(0);
            *compte += 1;
        }

        // Nous en tirons une table des choix face auxquels on peut se
        // retrouver, avec la probabilité de chacun.
        let norme = 1. / (nb_comb as Flottant);
        let mut choix_et_probas =
            comptage_choix.into_iter()
                          .map(|(choix, nb)| (choix, (nb as Flottant) * norme))
                          .collect::<Vec<ChoixEtProba>>();

        // Il vaut mieux trier cette table, ça simplifie la lecture et met la
        // combinaison perdante au début.
        choix_et_probas.sort_unstable_by(|(tirage1, _), (tirage2, _)| tirage1.cmp(tirage2));
        let proba_perte = choix_et_probas[0].1;

        // Nous pouvons maintenant énumérer les combinaisons possibles
        println!("Choix possibles: {}", choix_et_probas.len());
        for (choix, proba) in choix_et_probas.iter() {
            println!("- {:?} (Proportion: {})", choix, proba);
        }

        // ...et l'espérance sans relance
        let esperance_sans_relance : Flottant =
            choix_et_probas.iter()
                .map(|(choix, proba)| {
                    choix.iter()
                         .map(|comb| comb.valeur())
                         .max()
                         .unwrap_or(0) as Flottant
                    * proba
                })
                .sum();
        println!("Espérance sans relance: {}", esperance_sans_relance);

        // Gardons de côté les choix face auxquels on peut se retrouver (et leur
        // proba) pour ce nombre de dés.
        stats_jets.push(StatsJet {
            choix_et_probas,
            esperance_sans_relance,
            proba_perte,
        });

        println!();
    }

    println!("=== ORDRE 1: FAUT-IL RELANCER? ===\n");

    for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
        let nb_des = idx_nb_des + 1;

        let mut nouvelle_esperance_min = 0.;

        println!("Cas à {} dés", nb_des);
        for (choix, proba) in stats.choix_et_probas.iter() {
            println!("- Choix: {:?} (Proba: {})", choix, proba);
            let mut val_sans_relance = 0;
            let mut esperance_min_sans_solde: Flottant = 0.;
            for comb in choix {
                println!("  * Combinaison: {:?}", comb);
                println!("    o Valeur sans relance: Solde + {}", comb.valeur());
                val_sans_relance = val_sans_relance.max(comb.valeur());
                let des_restants = nb_des - comb.nb_des();
                let nouv_nb_des = if des_restants == 0 { 6 } else { des_restants };
                let stats_nouv_des = &stats_jets[nouv_nb_des-1];
                let proba_gain = 1. - stats_nouv_des.proba_perte;
                println!("    o Nouveau nombre de dés: {} (Probabilité de gain: {}, Espérance >= {})",
                         nouv_nb_des, proba_gain, stats_nouv_des.esperance_sans_relance);
                let valeur_amortie = comb.valeur() as Flottant * proba_gain;
                println!("    o Espérance avec relance: Solde * {} + {} + Espérance({} dés | Solde=0)",
                         proba_gain, valeur_amortie, nouv_nb_des);
                let borne_inf_sans_solde = valeur_amortie + stats_nouv_des.esperance_sans_relance;
                println!("    o ...où {} + Espérance({} dés | Solde=0) >= {}",
                         valeur_amortie, nouv_nb_des, borne_inf_sans_solde);
                esperance_min_sans_solde = esperance_min_sans_solde.max(borne_inf_sans_solde);
            }
            println!("  * Dans le cas Solde = 0...");
            println!("    o Valeur sans relance: {}", val_sans_relance);
            println!("    o Espérance avec relance >= {}", esperance_min_sans_solde);
            if esperance_min_sans_solde > val_sans_relance as Flottant {
                println!("    o Il faut toujours relancer!");
                nouvelle_esperance_min += esperance_min_sans_solde * proba;
            } else {
                println!("    o On ne peut pas conclure pour l'instant...");
                nouvelle_esperance_min += val_sans_relance as Flottant * proba;
            }
        }
        println!("- L'espérance min passe de {} à {}", stats.esperance_sans_relance, nouvelle_esperance_min);
        // TODO: Modifier l'espérance sans relance et itérer
        println!();
    }

    // TODO: Que faire ensuite?
}
