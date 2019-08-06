use crate::NB_FACES;
use std::fmt::{self, Debug};


// Valeur d'une combinaison
pub type Valeur = u16;

// Valeur minimale d'un dé
pub const VALEUR_MIN_DE : Valeur = 50;

// Combinaison gagnante définie par la règle de la mitraillette, que l'on peut
// choisir d'encaisser ou de mettre de côté en relançant le reste des dés.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl Debug for Combinaison {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use Combinaison::*;
        match *self {
            Suite => write!(formatter, "Suite"),
            TriplePaire => write!(formatter, "3Paires"),
            BrelanDouble { idx_faces } => {
                write!(formatter, "Brelan{}+Brelan{}",
                       idx_faces[0]+1, idx_faces[1]+1)
            },
            BrelanSimple { idx_face, nb_un, nb_cinq } => {
                write!(formatter, "Brelan{}", idx_face+1)?;
                if nb_un > 0 { write!(formatter, "+{}x1", nb_un)?; }
                if nb_cinq > 0 { write!(formatter, "+{}x5", nb_cinq)?; }
                Ok(())
            },
            FacesSimples { nb_un, nb_cinq } => {
                if nb_un > 0 {
                    write!(formatter, "{}x1", nb_un)?;
                    if nb_cinq > 0 { write!(formatter, "+")?; }
                }
                if nb_cinq > 0 { write!(formatter, "{}x5", nb_cinq)?; }
                Ok(())
            },
        }
    }
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
    pub fn nb_des(&self) -> usize {
        use Combinaison::*;
        match self {
            Suite | TriplePaire | BrelanDouble { .. } => 6,
            BrelanSimple { idx_face: _, nb_un, nb_cinq } => 3 + nb_un + nb_cinq,
            FacesSimples { nb_un, nb_cinq } => nb_un + nb_cinq,
        }
    }
}