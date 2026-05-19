#[cfg(test)]
mod tests {
    use crate::domain::{FragmentDefinition, Molecule, Element, Atom, Bond};
    use crate::fragments::list_available_fragments;
    use serde_json::json;

    #[test]
    fn test_fragment_definition_deserialization() {
        let json_data = json!({
            "name": "methyl",
            "displayName": "Methyl Group",
            "description": "A CH3 methyl group fragment.",
            "templateName": "methane",
            "molecule": {
                "name": "test",
                "atoms": [
                    { "id": 1, "element": "C", "position": [0.0, 0.0, 0.0] }
                ],
                "bonds": [
                    { "id": 1, "atomIds": [1, 2], "order": 1 }
                ]
            },
            "attachPorts": []
        });

        let def: FragmentDefinition = serde_json::from_value(json_data).unwrap();
        
        assert_eq!(def.name, "methyl");
        assert_eq!(def.molecule.name, "test");
        assert_eq!(def.molecule.atoms.len(), 1);
        assert_eq!(def.molecule.atoms[0].element, Element::C);
    }

    #[test]
    fn test_load_fragments_from_files() {
        // Run from src-tauri, assumes running in project root
        let fragments = list_available_fragments();
        assert!(!fragments.is_empty(), "Should load at least one fragment");
        
        let methyl = fragments.iter().find(|f| f.name == "methyl").expect("Should find methyl fragment");
        assert_eq!(methyl.template_name, "methane");
        assert_eq!(methyl.molecule.name, "methane");
        assert!(methyl.molecule.atoms.len() > 0);
    }
}
