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


fn main() {
    println!();

    // On étudie des lancers de 1 à 6 dés
    for nb_des in 1..=NB_DES_TOT {
        let nb_comb = NB_FACES.pow(nb_des as u32);
        println!("Nombre de lancers possibles à {} dés: {}", nb_des, nb_comb);

        let mut val_totale = 0;
        let mut nb_par_tirage: HashMap<Choix, u16> = HashMap::new();

        // On énumère tous les lancers possibles pour ce nombre de dés
        for num_comb in 0..nb_comb {
            let mut reste = num_comb;
            let mut histo = [0; NB_FACES];

            // On énumère les faces en traitant la combinaison comme un nombre
            // en base NB_FACES (note: la face 1 est numérotée 0), et on calcule
            // l'histogramme du nombre de dés étant tombé sur chaque face.
            /* print!("- Combinaison: "); */
            for _ in 0..nb_des {
                let idx_face = reste % NB_FACES;
                histo[idx_face] += 1;
                reste /= NB_FACES;
            }

            // On énumère les combinaisons qu'on peut prendre
            let choix = choix::enumerer_choix(histo);
            let nb = nb_par_tirage.entry(choix.clone()).or_insert(0);
            *nb += 1;

            // Supposons qu'on s'arrête là, combien gagne-t'on?
            let valeur_max = choix.iter().map(|comb| comb.valeur()).max();
            if let Some(valeur_max) = valeur_max {
                val_totale += valeur_max;
            }

            // TODO: Calculer le score de l'interprétation de valeur maximale de
            //       la main. En la sommant entre les différentes combinaisons
            //       dans un accumulateur u64, on pourra alors déterminer
            //       l'espérance de gain "à l'ordre 1", c'est à dire dans le
            //       cadre d'une stratégie simpliste où on lance les dés une
            //       seule fois et s'arrête immédiatement.
            //
            //       Cette espérance de gain approximative pourra ensuite
            //       être raffinée en parcourant l'arbre de décision à
            //       profondeur N, utilisant l'approximation d'ordre 1 comme
            //       estimateur de l'espérance en fin de parcours. Par exemple,
            //       on peut calculer l'espérance de gain à deux jets de dés en
            //       calculant le gain attendu pour chaque décision possible
            //       d'un tour réussi selon les dés qu'on choisit ou non de
            //       prendre, en supposant que le jet des dés restants suit
            //       l'approximation d'ordre 1.
            //
            //       Puisqu'un tour de mitraillette se termine quasiment
            //       toujours en une dizaine de tours grand max, un raffinement
            //       à ordre fini et pas trop grand devrait fournir une
            //       excellente approximation de l'espérance de gain réelle et
            //       donner une piste vers la stratégie optimale.

            // Certains résultats peuvent se comptabiliser de plusieurs façons:
            // - Une suite (500 points) contient un as et un cinq (150 points)
            // - Une triple paire peut contenir un ou deux brelans si il y a des
            //   paires identiques, et peut contenir un certain nombre de paires
            //   de 1 ou de 5.
            // - Un brelan peut contenir des 1 ou des 5, mais vaut toujours plus
            //   que la somme de ses constituants car triple 1 = 1000 et pas 100

            // Une triple paire contenant des brelans coûte parfois plus cher si
            // on l'interprète en tant que brelan. En cas de double brelan,
            // l'interprétation "brelan" est favorable sauf en cas de brelan
            // de 2. En cas de brelan simple, l'interprétation "brelan" est
            // équivalente (sur le seul critère du score) pour un brelan de 5
            // et favorable pour un brelan de 6 ou de 1.

            // Il s'agit ici d'une évaluation sur le seul critère du score
            // maximal en ne relançant pas les dés. En ajoutant le critère du
            // nombre de dés consommé, la situation deviendra plus complexe.
        }

        // Nous pouvons maintenant énumérer les combinaisons possibles
        println!("Tirages possibles: {}", nb_par_tirage.len());
        for (tirage, &nb) in nb_par_tirage.iter() {
            let prop = nb as f32 / nb_comb as f32;
            println!("- {:?} (Proportion: {})", tirage, prop);
        }

        // ...ce qui permet de calculer la proportion de jets perdants
        let prop_perdant = nb_par_tirage[&Choix::new()] as f32 / nb_comb as f32;
        println!("Proportion combinaisons perdantes: {}", prop_perdant);

        // ...et l'espérance de gain à un jet de dé
        let esperance_un_jet = val_totale as f32 / nb_comb as f32;
        println!("Espérance de gain à un jet: {}", esperance_un_jet);

        // TODO: Calculer l'espérance de gain totale

        println!();
    }
}
