use crate::{
    Flottant,
    NB_DES_TOT,
    choix,
    combinaison::{Combinaison, Valeur},
};

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Debug},
};


// Ce qu'on sait sur les lancers de dés à la mitraillette
pub struct StatsDes {
    // Données pour chaque nombre de dés
    stats_jets: Box<[StatsJet]>,
}

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face si on tire des combinaisons
    stats_choix: Box<[StatsChoix]>,

    // On garde en cache l'espérance de gain pour un nombre de relances <= N
    // donné et une certaine mise initiale.
    esperance: RefCell<HashMap<(usize, Valeur), Flottant>>,
}

// L'un dex choix face auxquels un jet de dés peut nous placer
struct StatsChoix {
    // Combinaisons entre lesquels il faut choisir
    choix: Box<[Possibilite]>,

    // Probabilité qu'on a de faire face à ce choix
    proba: Flottant,
}

// L'une des possibilités entre lesquelles il faut alors choisir
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
        write!(formatter, "{:?} ({}pt, relance {}d)",
               self.comb, self.valeur, self.nb_des_relance)
    }
}

impl StatsJet {
    // Initialiser les statistiques à N dés
    pub fn new(nb_des: usize) -> Self {
        // On énumère les choix de combinaisons face auxquels on peut se
        // retrouver en lançant ce nombre de dés, et avec quelle probabilité.
        let mut choix_et_probas = choix::enumerer_choix(nb_des);

        // On retire le cas perdant, car il est spécial à plusieurs égards
        // (on perd la mise précédente, on ne peut pas choisir de continuer)
        choix_et_probas.remove(&[][..]);

        // Pour les autres choix, on note quelques compléments
        let stats_choix =
            choix_et_probas.into_iter()
                .map(|(choix, proba)| {
                    // Valeur de chaque combinaison, nombre de dés si on relance
                    let choix = choix.into_iter()
                        .map(|comb| {
                            let valeur = comb.valeur();
                            let des_restants = nb_des - comb.nb_des();
                            let nb_des_relance = if des_restants == 0 {
                                NB_DES_TOT
                            } else {
                                des_restants
                            };
                            Possibilite {
                                comb,
                                valeur,
                                nb_des_relance,
                            }
                        }).collect::<Box<[_]>>();

                    // ...et, bien sûr, on garde la proba de côté
                    StatsChoix {
                        choix,
                        proba,
                    }
                }).collect::<Box<[_]>>();

        // ...et avec ça on est paré
        Self {
            stats_choix,
            esperance: RefCell::new(HashMap::new()),
        }
    }
}

impl StatsDes {
    // Initialiser les calculs statistiques à la mitraillette
    pub fn new() -> Self {
        Self {
            stats_jets: (1..=NB_DES_TOT).map(StatsJet::new)
                                        .collect::<Box<[_]>>(),
        }
    }

    // Espérance de gain pour une stratégie qui la maximise, en partant d'un
    // certain nombre de dés et d'une certaine mise préalable
    pub fn esperance(&self, nb_des: usize, mise: Valeur) -> Flottant {
        let mut num_relances = 0;
        let mut ancienne_esperance = 0.;
        loop {
            let esperance = self.calcul_esperance(nb_des, mise, num_relances);
            assert!(esperance >= ancienne_esperance);
            if esperance == ancienne_esperance { return esperance; }
            ancienne_esperance = esperance;
            num_relances += 1;
        }
    }

    // Calcul de l'espérance de gain en s'autorisant à relancer les dés au plus
    // N fois (une profondeur de relance infinie n'est pas calculable).
    fn calcul_esperance(&self, nb_des: usize, mise: Valeur, max_relances: usize) -> Flottant {
        // Est-ce que, par chance, j'ai déjà étudié ce cas précédemment?
        let stats_jet = &self.stats_jets[nb_des-1];
        if let Some(&esperance_lancer) = stats_jet.esperance.borrow()
                                                  .get(&(max_relances, mise)) {
            return esperance_lancer;
        }

        // Le but est de déterminer une espérance de gain pour un certain lancer
        let mut esperance_lancer = 0.;

        // On passe en revue tous les résultats de lancers gagnants
        for stats_choix in stats_jet.stats_choix.iter() {
            // On peut empocher la combinaison de valeur maximale...
            let valeur_max = stats_choix.choix.iter()
                                              .map(|poss| poss.valeur)
                                              .max()
                                              .unwrap();

            // ...ou bien on peut relancer, de 1 à num_relances fois. Parmi ces
            // options, on cherche celle qui maximise l'espérance de gain.
            let mut esperance_max = (mise + valeur_max) as Flottant;

            // Pour traiter les relances, il suffit de considérer chaque combinaison
            // qu'on peut empocher, l'ajouter à la mise, et faire une récursion avec
            // le nouveau nombre de dés et un budget relance réduit
            for num_relances in 1..=max_relances {
                for poss in stats_choix.choix.iter() {
                    let esperance =
                        self.calcul_esperance(poss.nb_des_relance,
                                              mise + poss.valeur,
                                              num_relances - 1);
                    esperance_max = esperance_max.max(esperance);
                }
            }

            // A la fin, on pondère cette espérance maximale par la probabilité de
            // faire face au choix qu'on a considéré.
            esperance_lancer += esperance_max * stats_choix.proba;
        }

        // On met en cache ce résultat
        assert_eq!(stats_jet.esperance.borrow_mut()
                            .insert((max_relances, mise), esperance_lancer),
                   None);

        // On retourne ce résultat à l'appelant
        esperance_lancer
    }
}