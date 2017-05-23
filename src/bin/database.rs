use grafen::substrate::{LatticeType, SubstrateConf};
use grafen::system::{Atom, Coord, ResidueBase};
use serde_json;

struct DataBase {
    filename: Option<String>,
    residues: Vec<ResidueBase>,
    substrate_confs: Vec<SubstrateConfEntry>,
}

impl Default for DataBase {
    fn default() -> DataBase {
        const SP_GRAPHENE: f64 = 0.142;
        let res_graphene = resbase!("GRP", ("C", SP_GRAPHENE / 2.0, SP_GRAPHENE / 2.0, 0.0));

        const SP_SILICA: f64 = 0.45;
        const DZ_SILICA: f64 = 0.151;
        let res_silica = resbase!("SIO", ("O1", SP_SILICA / 4.0, SP_SILICA / 6.0, DZ_SILICA),
                                         ("SI", SP_SILICA / 4.0, SP_SILICA / 6.0, 0.0),
                                         ("O2", SP_SILICA / 4.0, SP_SILICA / 6.0, -DZ_SILICA));

        DataBase {
            filename: None,
            residues: vec![
                res_graphene.clone(),
                res_silica.clone()
            ],
            substrate_confs: vec![
                SubstrateConfEntry {
                    name: "Graphene".to_string(),
                    lattice: LatticeType::Hexagonal { a: SP_GRAPHENE },
                    residue: res_graphene.clone(),
                    std_z: None,
                },
                SubstrateConfEntry {
                    name: "Silica".to_string(),
                    lattice: LatticeType::Triclinic { a: SP_SILICA, b: SP_SILICA, gamma: 60.0 },
                    residue: res_silica.clone(),
                    std_z: None,
                },
            ],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SubstrateConfEntry {
    name: String,
    lattice: LatticeType,
    residue: ResidueBase,
    std_z: Option<f64>,
}

impl SubstrateConfEntry {
    fn to_conf(&self, size_x: f64, size_y: f64) -> SubstrateConf {
        SubstrateConf {
            lattice: self.lattice.clone(),
            residue: self.residue.clone(),
            size: (size_x, size_y),
            std_z: self.std_z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substrate_conf_entry_into_conf() {
        let (size_x, size_y) = (2.0, 3.0);
        let conf = SubstrateConfEntry {
                    name: "".to_string(),
                    lattice: LatticeType::Hexagonal { a: 1.0 },
                    residue: resbase!("RES", ("A", 0.0, 1.0, 2.0)),
                    std_z: Some(0.5),
        }.to_conf(size_x, size_y);

        assert_eq!(LatticeType::Hexagonal { a: 1.0 }, conf.lattice);
        assert_eq!(resbase!("RES", ("A", 0.0, 1.0, 2.0)), conf.residue);
        assert_eq!((size_x, size_y), conf.size);
        assert_eq!(Some(0.5), conf.std_z);
    }

    #[test]
    fn serialize_and_deserialize_residue_entry() {
        let base = resbase!(
            "RES",
            ("A1", 0.0, 1.0, 2.0),
            ("A2", 3.0, 4.0, 5.0)
        );

        let serialized = serde_json::to_string(&base).unwrap();
        let deserialized: ResidueBase = serde_json::from_str(&serialized).unwrap();

        assert_eq!(base, deserialized);
    }

    #[test]
    fn serialize_and_deserialize_substrateconf_entry() {
        let base = resbase!(
            "RES",
            ("A1", 0.0, 1.0, 2.0),
            ("A2", 3.0, 4.0, 5.0)
        );

        let mut conf = SubstrateConfEntry {
            name: "Test".to_string(),
            lattice: LatticeType::PoissonDisc{ density: 1.0 },
            residue: base,
            std_z: None,
        };

        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SubstrateConfEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(conf.name, deserialized.name);
        assert_eq!(conf.residue, deserialized.residue);
        assert_eq!(conf.std_z, deserialized.std_z);

        conf.std_z = Some(1.0);
        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SubstrateConfEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(conf.std_z, deserialized.std_z);
    }
}
