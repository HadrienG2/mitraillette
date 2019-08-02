use crate::NB_FACES;


// Valeur d'une combinaison (sommable sur toutes les combinaisons)
pub type Valeur = u64;

// Combinaison gagnante définie par la règle de la mitraillette, que l'on peut
// choisir d'encaisser ou de mettre de côté en relançant le reste des dés.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Combinaison {
    // 1 2 3 4 5 6
    Suite,

    // aa bb cc
    TriplePaire,

    // aaa bbb (trié par a < b pour éviter le double comptage)
    BrelanDouble { idx_faces: [usize; 2] },

    // aaa xyz (où x, y, z peut contenir 1 et 5)
    BrelanSimple { idx_face: usize, nb_un: usize, nb_cinq: usize },

    // Des 1, des 5, et rien d'autre
    FacesSimples { nb_un: usize, nb_cinq: usize },
}

impl Combinaison {
    // Valeur de la combinaison en points
    pub fn valeur(&self) -> Valeur {
        use Combinaison::*;
        const VALEURS_BRELANS: [Valeur; NB_FACES] = [1000, 200, 300, 400, 500, 600];
        match self {
            Suite | TriplePaire => 500,
            BrelanDouble { idx_faces: [idx_face_1, idx_face_2] } =>
                VALEURS_BRELANS[*idx_face_1] + VALEURS_BRELANS[*idx_face_2],
            BrelanSimple { idx_face, nb_un, nb_cinq } =>
                VALEURS_BRELANS[*idx_face]
                    + (*nb_un as Valeur) * 100
                    + (*nb_cinq as Valeur) * 50,
            FacesSimples { nb_un, nb_cinq } =>
                (*nb_un as Valeur) * 100
                    + (*nb_cinq as Valeur) * 50,
        }
    }

    // Nombre de dés consommé si on encaisse la combinaison
    #[allow(dead_code)]
    pub fn nb_des(&self) -> usize {
        use Combinaison::*;
        match self {
            Suite | TriplePaire | BrelanDouble { .. } => 6,
            BrelanSimple { idx_face: _, nb_un, nb_cinq } => 3 + nb_un + nb_cinq,
            FacesSimples { nb_un, nb_cinq } => nb_un + nb_cinq,
        }
    }
}