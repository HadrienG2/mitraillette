use crate::{
    Flottant,
    NB_DES_TOT,
    SCORE_MAX,
    Valeur,
    choix,
    combinaison::Combinaison,
};

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Debug},
};


// Ce qu'on sait sur les lancers de dés à la mitraillette
pub struct Stats {
    // Données pour chaque nombre de dés
    stats_jets: Box<[StatsJet]>,
}

// Ce qu'on sait sur le lancer d'un certain nombre de dés
struct StatsJet {
    // Choix auxquels on peut faire face si on tire des combinaisons
    stats_choix: Box<[StatsChoix]>,

    // On garde en cache l'espérance de gain pour un certain score de départ,
    // une mise qu'on possédait avant de lancer les dés, et un nombre de
    // relances maximal. Cela évite de recalculer plein de fois la même chose en
    // étudiant les relances de dés.
    esperance: RefCell<HashMap<(Valeur, Valeur, usize), Flottant>>,

    // Même topo avec la probabilité de finir la partie
    proba_fin: RefCell<HashMap<(Valeur, Valeur, usize), Flottant>>,
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
            proba_fin: RefCell::new(HashMap::new()),
        }
    }
}

impl Stats {
    // Initialiser les calculs statistiques à la mitraillette
    pub fn new() -> Self {
        Self {
            stats_jets: (1..=NB_DES_TOT).map(StatsJet::new)
                                        .collect::<Box<[_]>>(),
        }
    }

    // Gain moyen quand on risque "mise" points en lançant "nb_des" dés
    pub fn gain_moyen(&self,
                      score: Valeur,
                      nb_des: usize,
                      mise: Valeur) -> Flottant
    {
        self.esperance(score, nb_des, mise) - mise as Flottant
    }

    // Espérance de gain pour une stratégie qui la maximise, en partant d'un
    // certain nombre de dés et d'une certaine mise préalable
    pub fn esperance(&self,
                     score: Valeur,
                     nb_des: usize,
                     mise: Valeur) -> Flottant
    {
        let mut num_relances = 0;
        let mut ancienne_esperance = 0.;
        loop {
            let esperance = self.calcul_esperance(score, nb_des, mise, num_relances);
            assert!(esperance >= ancienne_esperance);
            if esperance == ancienne_esperance { return esperance; }
            ancienne_esperance = esperance;
            num_relances += 1;
        }
    }

    // Probabilité de gagner (atteindre 10000) en continuant à lancer les dés.
    //
    // Pour des scores faibles, les régions de l'arbre des lancer de dés où on
    // gagne sont très profondes, donc il vaut mieux s'arrêter à une certaine
    // profondeur quitte à sous-estimer cette probabilité.
    //
    pub fn proba_fin(&self,
                     score: Valeur,
                     nb_des: usize,
                     mise: Valeur,
                     max_relances: usize) -> Flottant
    {
        let mut ancienne_proba = 0.;
        for num_relances in 0..max_relances {
            let proba = self.calcul_proba_fin(score, nb_des, mise, num_relances);
            assert!(proba >= ancienne_proba);
            if proba > 0. && proba == ancienne_proba { print!("{} relances -- ", num_relances); return proba; }
            ancienne_proba = proba;
        }
        self.calcul_proba_fin(score, nb_des, mise, max_relances)
    }

    // Calcul de l'espérance de gain en s'autorisant à relancer les dés N fois
    fn calcul_esperance(&self,
                        score: Valeur,
                        nb_des: usize,
                        mise: Valeur,
                        max_relances: usize) -> Flottant
    {
        // Est-ce que, par chance, j'ai déjà étudié ce cas précédemment?
        let stats_jet = &self.stats_jets[nb_des-1];
        if let Some(&esperance_lancer) = stats_jet.esperance.borrow()
                                                  .get(&(score, mise, max_relances)) {
            return esperance_lancer;
        }

        // Le but est de déterminer une espérance de gain pour un certain lancer
        let mut esperance_lancer = 0.;

        // On passe en revue tous les résultats de lancers gagnants
        for stats_choix in stats_jet.stats_choix.iter() {
            // On note la valeur de la combinaison la plus chère. Si elle nous
            // amène à plus de 10000, on ne peut pas s'arrêter là.
            let valeur_max = stats_choix.choix.iter()
                                              .map(|poss| poss.valeur)
                                              .max()
                                              .unwrap();
            let arret_possible = score + mise + valeur_max <= SCORE_MAX;

            // On cherche la stratégie qui maximise l'espérance
            let mut esperance_max : Flottant = 0.;

            // On considère la possibilité de prendre chaque combinaison...
            for poss in stats_choix.choix.iter() {
                let nouvelle_mise = mise + poss.valeur;

                // Si la règle nous y autorise, on peut s'arrêter là
                if arret_possible {
                    esperance_max = esperance_max.max(nouvelle_mise as Flottant);
                }

                // Si prendre cette combinaison ne nous fait pas atteindre ou
                // dépasser le score maximal, on peut aussi relancer <= N fois
                if score + nouvelle_mise >= SCORE_MAX { continue; }
                for num_relances in 1..=max_relances {
                    let esperance =
                        self.calcul_esperance(score,
                                              poss.nb_des_relance,
                                              nouvelle_mise,
                                              num_relances - 1);
                    esperance_max = esperance_max.max(esperance);
                }
            }

            // A la fin, on pondère l'espérance maximale calculée par la
            // probabilité de faire face au choix qu'on a considéré
            esperance_lancer += esperance_max * stats_choix.proba;
        }

        // On met en cache ce résultat
        assert_eq!(stats_jet.esperance.borrow_mut()
                            .insert((score, mise, max_relances), esperance_lancer),
                   None);

        // On retourne ce résultat à l'appelant
        esperance_lancer
    }

    // Calcul de la probabilité de gagner la partie avec N relances
    fn calcul_proba_fin(&self,
                        score: Valeur,
                        nb_des: usize,
                        mise: Valeur,
                        max_relances: usize) -> Flottant
    {
        // Est-ce que, par chance, j'ai déjà étudié ce cas précédemment?
        let stats_jet = &self.stats_jets[nb_des-1];
        if let Some(&proba_fin_partie) = stats_jet.proba_fin.borrow()
                                                  .get(&(score, mise, max_relances)) {
            return proba_fin_partie;
        }

        // Le but est de déterminer la probabilité de gagner la partie
        let mut proba_fin_partie = 0.;

        // On passe en revue tous les résultats de lancers gagnants
        for stats_choix in stats_jet.stats_choix.iter() {
            // On note la valeur de la combinaison la plus chère
            let valeur_max = stats_choix.choix.iter()
                                              .map(|poss| poss.valeur)
                                              .max()
                                              .unwrap();

            // Si elle nous amène à 10000, on a gagné
            let mut proba_fin_max : Flottant =
                if score + mise + valeur_max == 10000 { 1. } else { 0. };

            // Sinon, on peut tenter de prendre une combinaison qui nous amène
            // à mons de 10000 et relancer.
            for poss in stats_choix.choix.iter() {
                let nouvelle_mise = mise + poss.valeur;
                if score + nouvelle_mise >= SCORE_MAX { continue; }
                for num_relances in 1..=max_relances {
                    let proba_fin =
                        self.calcul_proba_fin(score,
                                              poss.nb_des_relance,
                                              nouvelle_mise,
                                              num_relances - 1);
                    proba_fin_max = proba_fin_max.max(proba_fin);
                }
            }

            // On pondère le résultat par la chance de tirer ce jet
            proba_fin_partie += proba_fin_max * stats_choix.proba;
        }

        // On met en cache ce résultat
        assert_eq!(stats_jet.proba_fin.borrow_mut()
                            .insert((score, mise, max_relances), proba_fin_partie),
                   None);

        // On retourne ce résultat à l'appelant
        proba_fin_partie
    }
}