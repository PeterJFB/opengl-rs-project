use std::path::Path;

pub struct ObjReader {}

pub enum ObjEntry {
    Vertex,
    Face,
    Comment,
}

impl ObjEntry {
    fn from_code(code: &str) -> Result<ObjEntry, String> {
        match code {
            "v" => Ok(ObjEntry::Vertex),
            "f" => Ok(ObjEntry::Face),
            "#" => Ok(ObjEntry::Comment),
            e => Err(e.to_string()),
        }
    }
}

impl ObjReader {
    pub fn read(obj_path: &str) -> (Vec<f32>, Vec<u32>) {
        // Mainly inspired from shared_generator
        let path = Path::new(obj_path);

        // Attempt to retrieve extension
        if let Some(extension) = path.extension() {
            // See if extension is obj
            match extension.to_str().expect("Failed to read extension.") {
                "obj" => Ok(()),
                e => Err(e.to_string()),
            }
            .expect("Failed to parse extension.");

            // If obj, read rest of file
            let obj_src = std::fs::read_to_string(path)
                .expect(&format!("Failed to read shader source. {}", obj_path));

            // Pass string-data to dedicated parser and return it
            ObjReader::parse_obj_src(&obj_src)
        } else {
            panic!("Failed to read extension of file with path: {}", obj_path);
        }
    }

    /**
     * Assumes winding of faces are consistent (i.e. does not use normals).
     */
    pub fn parse_obj_src(obj_src: &str) -> (Vec<f32>, Vec<u32>) {
        let mut verticies = vec![];
        let mut indicies = vec![];

        // Iterate over each entry
        for line in obj_src.lines() {
            let lexemes: Vec<&str> = line.split_whitespace().collect();

            // Map string code to either vertex or face and append data to corresponding vector
            match ObjEntry::from_code(&lexemes[0]) {
                Ok(ObjEntry::Vertex) => verticies.append(
                    &mut lexemes[1..]
                        .iter()
                        .map(|&lex| lex.parse().unwrap())
                        .collect(),
                ),
                Ok(ObjEntry::Face) => indicies.append(
                    &mut lexemes[1..]
                        .iter()
                        .map(|&lex| lex.parse().unwrap())
                        .collect(),
                ),
                Ok(ObjEntry::Comment) => (),
                Err(s) => {
                    eprintln!("Unknown entry: {}", s)
                }
            }
        }

        (verticies, indicies)
    }
}
