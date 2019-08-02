const NB_DES_TOT : usize = 6;
const NB_FACES : usize = 6;

type Face = u8;
type Histogramme = [usize; NB_FACES];

// Combinaisons gagnantes définies par la règle de la mitraillette
#[derive(Debug)]
enum Combinaisons {
    // Consomme 6 dés, vaut 500 points
    Suite,

    // Consomme 6 dés, vaut 500 points
    TriplePaire,

    // Consomme 6 dés, vaut la somme des valeurs de brelans simples, peut être
    // interprété comme un brelan simple
    BrelanDouble { faces: [Face; 2] },

    // Consomme 3 dés + nb 1 et 5, vaut (face * 100) ou 1000 si face = 1,
    // plus la valeur des uns et des cinq
    BrelanSimple { face: Face, nb_un: usize, nb_cinq: usize },

    // Consomme nb 1 et 5, vaut 100 par 1 et 50 par 5
    FacesSimples { nb_un: usize, nb_cinq: usize },
}


// Enumérer les combinaisons pour un histogramme donné
fn enumerer_combinaisons(histo: Histogramme, combis: &mut Vec<Combinaisons>) {
    // Traitement des suites
    if histo.iter().all(|&bin| bin == 1) {
        combis.push(Combinaisons::Suite);
    }

    // Traitement des triple paires
    let num_paires: usize = histo.iter().map(|&bin| bin/2).sum();
    if num_paires == 3 {
        combis.push(Combinaisons::TriplePaire);
    }

    // Traitement des brelans
    for (idx_face, &bin) in histo.iter().enumerate() {
        if bin < 3 { continue; }

        let face = (idx_face as Face) + 1;
        combis.push(Combinaisons::BrelanSimple { face, nb_un: 0, nb_cinq: 0 });
        
        let mut combis_internes = Vec::new();
        let mut histo_sans_brelans = histo.clone();
        histo_sans_brelans[idx_face] -= 3;
        enumerer_combinaisons(histo_sans_brelans, &mut combis_internes);

        for combi in combis_internes {
            match combi {
                Combinaisons::BrelanSimple { face: face_2, nb_un: 0, nb_cinq: 0 } => {
                    if face_2 < face { continue; } // Evite le double comptage
                    combis.push(Combinaisons::BrelanDouble { faces: [face, face_2] });
                }
                Combinaisons::FacesSimples { nb_un, nb_cinq } => {
                    combis.push(Combinaisons::BrelanSimple { face, nb_un, nb_cinq });
                }
                _ => unreachable!()
            }
        }
    }

    // Traitement des faces simples
    // FIXME: Ignorer le cas où on prend un 1 quand on pourrait prendre un 5
    for nb_un in 0..=histo[0] {
        for nb_cinq in 0..=histo[4] {
            if nb_un == 0 && nb_cinq == 0 { continue; }
            combis.push(Combinaisons::FacesSimples{ nb_un, nb_cinq });
        }
    }
}


fn main() {
    // On étudie des lancers de 1 à 6 dés
    for nb_des in 1..=NB_DES_TOT {
        let nb_comb = NB_FACES.pow(nb_des as u32);
        println!("Nombre de lancers possibles à {} dés: {}", nb_des, nb_comb);

        let mut nb_gagnants = 0;

        // On énumère tous les lancers possibles pour ce nombre de dés
        for num_comb in 0..nb_comb {
            let mut reste = num_comb;
            let mut histo = [0; NB_FACES];

            // On énumère les faces en traitant la combinaison comme un nombre
            // en base NB_FACES (note: la face 1 est numérotée 0), et on calcule
            // l'histogramme du nombre de dés étant tombé sur chaque face.
            print!("- Combinaison: ");
            for _ in 0..nb_des {
                let idx_face = reste % NB_FACES;
                print!("{}", idx_face+1);
                histo[idx_face] += 1;
                reste /= NB_FACES;
            }
            println!(" (histogramme: {:?})", histo);

            // Enumérons les différentes manières de lire le résultat selon les
            // règles de la mitraillette.
            let mut combis = Vec::new();
            enumerer_combinaisons(histo, &mut combis);

            for combi in combis.iter() {
                println!("  * {:?}", combi);
            }

            // On comptabilise les tirages gagnants...
            if combis.len() > 0 {
                nb_gagnants += 1;
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

        // ...ce qui permet de calculer la proportion de jets perdants
        let prop_perdant = (nb_comb - nb_gagnants) as f32 / nb_comb as f32;
        println!("Proportion combinaisons perdantes: {}", prop_perdant);

        // TODO: Calculer l'espérance de gain
    }
}
