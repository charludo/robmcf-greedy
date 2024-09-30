use super::{Network, Vertex};
use crate::Result;

impl Network {
    pub fn to_latex(&self, filename: &str, no_text: bool) -> Result<()> {
        let vertices = self.normalize_vertex_positions();
        let mut latex = Vec::new();

        latex.push(
            "\\begin{figure}[t]
	            \\centering
	            \\begin{tikzpicture}[>=stealth, auto, node distance=2cm, thick]"
                .to_string(),
        );

        for (i, vertex) in vertices.iter().enumerate() {
            latex.push(format!(
                "\\node[circle, draw] (v{i}) at ({},{}) {{{}}};",
                vertex.x,
                vertex.y,
                if no_text { "" } else { &vertex.name }
            ));
        }

        for (i, j) in self.capacities.indices() {
            if *self.capacities.get(i, j) == 0 && !self.fixed_arcs.contains(&(i, j)) {
                continue;
            }
            latex.push(format!(
                "\\draw[->{}] (v{i}) to[bend left=20] node[below, sloped] {{\\footnotesize{{${}$}}}} node[above, sloped] {{\\footnotesize{{${}$}}}} (v{j});",
                if self.fixed_arcs.contains(&(i, j)) { ", draw=teal" } else { "" },
                if no_text { " ".to_string() } else { self.costs.get(i, j).to_string() },
                if no_text { " ".to_string() } else { self.capacities.get(i, j).to_string() },
            ));
        }

        latex.push(
            "\\end{tikzpicture}
	        \\caption{TODO.}
	        \\label{fig:TODO}
        \\end{figure}"
                .to_string(),
        );

        let latex_str = latex.join("\n");

        log::debug!("Writing\n{latex_str}\nto {filename}");
        std::fs::write(filename, latex_str)?;
        Ok(())
    }

    fn normalize_vertex_positions(&self) -> Vec<Vertex> {
        let mut min_x = 0.0;
        let mut max_x = 0.0;
        let mut min_y = 0.0;
        let mut max_y = 0.0;

        for vertex in &self.vertices {
            if vertex.x < min_x {
                min_x = vertex.x;
            }
            if vertex.x > max_x {
                max_x = vertex.x;
            }
            if vertex.y < min_y {
                min_y = vertex.y;
            }
            if vertex.y > max_y {
                max_y = vertex.y;
            }
        }

        let width = 12.0;
        let height = width * (max_y - min_y) / (max_x - min_x);

        self.vertices
            .iter()
            .map(|v| Vertex {
                name: v.name.clone(),
                is_station: v.is_station,
                x: if max_x == 0. && min_x == 0.0 {
                    0.0
                } else {
                    ((v.x - min_x) / (max_x - min_x)) * width
                },
                y: if max_y == 0.0 && min_y == 0.0 {
                    0.0
                } else {
                    ((v.y - min_y) / (max_y - min_y)) * height
                },
            })
            .collect::<Vec<Vertex>>()
    }
}
