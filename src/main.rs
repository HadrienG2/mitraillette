mod choix;
mod combinaison;

use crate::combinaison::{Combinaison, Valeur};

use std::{
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
#[derive(Eq, PartialEq, PartialOrd, Ord)]
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

        // Nous en tirons une table des choix face auxquels on peut se
        // retrouver, avec la probabilité de chacun et la valeur de la
        // combinaison la plus chère.
        let norme = 1. / (nb_comb as Flottant);
        let mut stats_choix =
            comptage_choix.into_iter()
                .map(|(choix, compte)| {
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
                    let valeur_max = choix.iter()
                                          .map(|poss| poss.valeur)
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
            let mut esperance_min_sans_solde: Flottant = 0.;

            // Pour cela, on étudie les relances pour chaque combinaison...
            for poss in stat_choix.choix.iter() {
                // Voyons les stats pour le nombre de dés qu'on relancerait
                let stats_nouv_des = &stats_jets[poss.nb_des_relance-1];

                // La probabilité de jet gagnant nous dit quelle proportion
                // de la valeur on garde en moyenne en relançant
                let valeur_amortie = poss.valeur as Flottant * stats_nouv_des.proba_gain;

                // On y ajoute notre borne inférieure de ce qu'on espère
                // gagner en relançant les dés restants
                let esperance_min = stats_nouv_des.esperance_gain_sans_relancer;

                // La somme est une borne inférieure de l'espérance de gain
                // si on prend cette combinaison et la relance
                let borne_inf_sans_solde = valeur_amortie + esperance_min;

                // On prend le maximum de ces bornes inférieures sur tous
                // les choix de combinaisons possibles
                esperance_min_sans_solde = esperance_min_sans_solde.max(borne_inf_sans_solde);

                // TODO: Stocker ces résultats intermédiaires pour permettre l'évaluation des relances doubles?
            }

            // On en déduit si il faut clairement relancer, et on intègre le
            // résultat maximal (valeur max si on s'arrête ou borne
            // inférieure de l'espérance de gain en cas de relance)
            let borne_inf_gain = esperance_min_sans_solde.max(stat_choix.valeur_max as Flottant);
            esperance_gain_relance_unique += borne_inf_gain * stat_choix.proba;
        }

        println!("Probabilité de perdre à {} dés: {}",
                 idx_nb_des + 1, 1. - stats.proba_gain);
        println!("Espérance de gain avec relance unique: {}", esperance_gain_relance_unique);
        println!("Lancer clairement pertinent si solde préalable < {}",
                 esperance_gain_relance_unique / (1. - stats.proba_gain));
        println!();
    }

    // TODO: Prendre en compte les relances doubles, triples...
}
