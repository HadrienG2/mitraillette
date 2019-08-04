use crate::{
    Flottant,
    NB_FACES,
    combinaison::Combinaison,
};

use std::collections::HashMap;


// Enumérer les choix auxquels on peut faire face en lançant N dés, et leurs
// probas. Le choix [] correspond à une absence de combinaisons (perdu!)
pub fn enumerer_choix(nb_des: usize) -> HashMap<Vec<Combinaison>, Flottant> {
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
        let choix = enumerer_combinaisons(histo);

        // ...et on en compte les occurences, dont on déduira la probabilité
        let compte = comptage_choix.entry(choix.clone()).or_insert(0);
        *compte += 1;
    }

    // Nous déduisons une table des choix auxquels on peut faire face, et du
    // nombre de tirages associés. On transforme les comptes en probabilités.
    let norme = 1. / (nb_comb as Flottant);
    comptage_choix.into_iter()
        .map(|(choix, compte)| (choix, compte as Flottant * norme))
        .collect()
}

// Histogramme d'un jet de dé par face (nb de dés tombé sur chaque face)
type HistogrammeFaces = [usize; NB_FACES];

// Combinaisons qu'on peut raisonnablement choisir pour un histogramme donné
fn enumerer_combinaisons(histo: HistogrammeFaces) -> Vec<Combinaison> {
    // Préparation du stockage
    let mut choix = Vec::new();

    // Traitement des suites
    if histo.iter().all(|&bin| bin == 1) {
        choix.push(Combinaison::Suite);
    }

    // Traitement des triple paires
    let num_paires: usize = histo.iter().map(|&bin| bin/2).sum();
    if num_paires == 3 {
        choix.push(Combinaison::TriplePaire);
    }

    // Traitement des brelans
    for (idx_face, &bin) in histo.iter().enumerate() {
        if bin < 3 { continue; }

        choix.push(Combinaison::BrelanSimple { idx_face, nb_un: 0, nb_cinq: 0 });
        
        let mut histo_sans_brelans = histo.clone();
        histo_sans_brelans[idx_face] -= 3;
        let choix_internes = enumerer_combinaisons(histo_sans_brelans);

        for combi in choix_internes {
            match combi {
                Combinaison::BrelanSimple { idx_face: idx_face_2, nb_un: 0, nb_cinq: 0 } => {
                    if idx_face_2 < idx_face { continue; } // Evite le double comptage
                    choix.push(Combinaison::BrelanDouble { idx_faces: [idx_face, idx_face_2] });
                }
                Combinaison::FacesSimples { nb_un, nb_cinq } => {
                    choix.push(Combinaison::BrelanSimple { idx_face, nb_un, nb_cinq });
                }
                _ => unreachable!()
            }
        }
    }

    // Traitement des faces simples, selon certains principes:
    // - Il n'est pas rationnel de prendre un 5 sans avoir pris tous les 1
    // - Il n'est pas rationnel de compter trois 5 ou 1 autrement que comme brelan
    // FIXME: Il faudra prendre en compte ces combinaisons lors de l'étude de
    //        l'aterrissage.
    for nb_un in 1..=(histo[0]%3) {
        choix.push(Combinaison::FacesSimples{ nb_un, nb_cinq: 0 });
    }
    for nb_cinq in 1..=(histo[4]%3) {
        choix.push(Combinaison::FacesSimples{ nb_un: histo[0]%3, nb_cinq });
    }

    // ...et on a tout traité
    choix
}