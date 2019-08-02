use crate::{
    HistogrammeFaces,
    combinaison::Combinaison,
};


// Pour un jet de dé possible, identifié par son histogramme, le joueur peut
// avoir le choix entre plusieurs combinaisons.
//
// 0 combinaison = fin du tour, perte des gains
// 1 combinaison = choix entre continuer et s'arrêter
// 2+ combinaisons = choix entre les combinaisons pour continuer (ou arrêt)
//
pub type Choix = Vec<Combinaison>;

// Enumérons les choix possibles pour un histogramme donné
pub fn enumerer_choix(histo: HistogrammeFaces) -> Choix {
    // Préparation du stockage
    let mut choix = Choix::new();

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
        let choix_internes = enumerer_choix(histo_sans_brelans);

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
    for nb_un in 1..=(histo[0]%3) {
        choix.push(Combinaison::FacesSimples{ nb_un, nb_cinq: 0 });
    }
    for nb_cinq in 1..=(histo[4]%3) {
        choix.push(Combinaison::FacesSimples{ nb_un: histo[0]%3, nb_cinq });
    }

    // ...et on a tout traité
    choix
}