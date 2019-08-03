mod choix;
mod combinaison;

use crate::choix::Choix;

use std::{
    cell::Cell,
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
type ChoixEtProba = (Choix, Flottant);

// Ce qu'on sait sur les jets d'un certain nombre de dés
struct StatsJet {
    choix_et_probas: Vec<ChoixEtProba>,
    min_esperance_gain: Cell<Flottant>,
    proba_perte: Flottant,
}


fn main() {
    println!("\n=== JETS ISOLES ===\n");

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
        // retrouver, avec la probabilité de chacun.
        let norme = 1. / (nb_comb as Flottant);
        let mut choix_et_probas =
            comptage_choix.into_iter()
                          .map(|(choix, nb)| (choix, (nb as Flottant) * norme))
                          .collect::<Vec<ChoixEtProba>>();

        // On trie la table, ce qui la rend plus lisible pour l'affichage, et
        // place la combinaison perdante au début. On récupère la probabilité de
        // celle-ci, puis on l'écarte puisqu'elle est spéciale (pas de choix)
        choix_et_probas.sort_unstable_by(|(tirage1, _), (tirage2, _)| tirage1.cmp(tirage2));
        let proba_perte = choix_et_probas.remove(0).1;
        println!("Probabilité de perte: {}", proba_perte);

        // On peut aussi calculer l'espérance de gain sans relance (on lance les
        // dés, on prend la combinaison la plus élevée, et on s'arrête là).
        //
        // C'est une borne inférieure de l'espérance de gain, puisqu'on ne
        // relancera pour gagner plus que si la relance rapporte en moyenne plus
        // que le gain maximal obtenu en s'arrêtant à cette combinaison.
        //
        let esperance_gain_sans_relancer : Flottant =
            choix_et_probas.iter()
                .map(|(choix, proba)| {
                    choix.iter()
                         .map(|comb| comb.valeur())
                         .max()
                         .unwrap_or(0) as Flottant
                    * proba
                })
                .sum();
        println!("Espérance sans relancer: {}", esperance_gain_sans_relancer);

        // Pour terminer, on énumère les combinaisons possibles
        println!("Choix auxquels on peut faire face:");
        for (choix, proba) in choix_et_probas.iter() {
            println!("- {:?} (Probabilité: {})", choix, proba);
        }

        // Nous gardons de côté ces calculs, on a besoin de les avoir effectués
        // pour tous les nombres de dés avant d'aller plus loin.
        stats_jets.push(StatsJet {
            choix_et_probas,
            min_esperance_gain: Cell::new(esperance_gain_sans_relancer),
            proba_perte,
        });

        println!();
    }

    println!("=== RELANCES EVIDENTES ===\n");

    // Maintenant, on peut affiner notre borne inférieure de l'espérance de gain
    // par récurence, en utilisant la borne inférieure 
    loop {
        let mut continuer = false;

        // Pour chaque nombre de dés...
        for (idx_nb_des, stats) in stats_jets.iter().enumerate() {
            let nb_des = idx_nb_des + 1;

            // ...on veut affiner notre borne inférieure de l'espérance de gain
            let ancienne_esperance_min = stats.min_esperance_gain.get();
            let mut nouvelle_esperance_min = 0.;

            // On passe en revue tous les choix auxquels on peut faire face
            println!("Cas à {} dés", nb_des);
            for (choix, proba) in stats.choix_et_probas.iter() {
                println!("- Choix: {:?} (Proba: {})", choix, proba);

                // Pour chaque choix, on examine la combinaison de plus forte
                // valeur, et on cherche une borne inférieure à l'espérance de
                // gain en cas de relance, sans solde préalable.
                let mut val_sans_relance = 0;
                let mut esperance_min_sans_solde: Flottant = 0.;

                // Pour cela, on énumère les combinaisons...
                for comb in choix {
                    println!("  * Combinaison: {:?}", comb);
                    println!("    o Valeur sans relance: Solde + {}", comb.valeur());
                    val_sans_relance = val_sans_relance.max(comb.valeur());
                    let des_restants = nb_des - comb.nb_des();
                    let nouv_nb_des = if des_restants == 0 { 6 } else { des_restants };
                    let stats_nouv_des = &stats_jets[nouv_nb_des-1];
                    let proba_gain = 1. - stats_nouv_des.proba_perte;
                    let esperance_min = stats_nouv_des.min_esperance_gain.get();
                    println!("    o Nouveau nombre de dés: {} (Probabilité de gain: {}, Espérance >= {})",
                             nouv_nb_des, proba_gain, esperance_min);
                    let valeur_amortie = comb.valeur() as Flottant * proba_gain;
                    println!("    o Espérance avec relance: Solde * {} + {} + Espérance({} dés | Solde=0)",
                             proba_gain, valeur_amortie, nouv_nb_des);
                    let borne_inf_sans_solde = valeur_amortie + esperance_min;
                    println!("    o ...où {} + Espérance({} dés | Solde=0) >= {}",
                             valeur_amortie, nouv_nb_des, borne_inf_sans_solde);
                    esperance_min_sans_solde = esperance_min_sans_solde.max(borne_inf_sans_solde);
                }

                // On voit que l'espérance à solde nulle est importante
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

            if nouvelle_esperance_min > ancienne_esperance_min {
                println!("- L'espérance minimale passe de {} à {}",
                         ancienne_esperance_min, nouvelle_esperance_min);
                stats.min_esperance_gain.set(nouvelle_esperance_min);
                continuer = true;
            } else if nouvelle_esperance_min == ancienne_esperance_min {
                println!("- L'espérance min est stable");
            } else {
                unreachable!();
            }

            println!();
        }

        if continuer {
            println!("------\n");
        } else {
            break;
        }
    }

    // TODO: Que faire ensuite?
}
