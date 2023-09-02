use glm::pi;

extern crate nalgebra_glm as glm;

pub struct ShapeGenerator {}

impl ShapeGenerator {
    pub fn generate_n_force(
        n_triangle_rows: usize,
        width: f32,
        height: f32,
    ) -> (Vec<f32>, Vec<u32>) {
        let t_width = width / n_triangle_rows as f32;
        let t_height = height / n_triangle_rows as f32;

        let n_triangles = n_triangle_rows * (n_triangle_rows + 1) / 2;
        let n_vertex_rows = n_triangle_rows + 1;
        let n_verticies = n_vertex_rows * (n_vertex_rows + 1) / 2;
        let mut verticies = vec![0.; n_verticies * 3];
        let mut indicies = vec![0; n_triangles * 3];

        let vertex_row_start_y = height / 2.;
        for vertex_row in 0..n_vertex_rows {
            let vertex_row_f = vertex_row as f32;
            let vertex_row_y = vertex_row_start_y - vertex_row_f * t_height;

            let vertex_row_start_x = -t_width * vertex_row_f / 2.;

            let vertex_row_index = if vertex_row == 0 {
                0
            } else {
                (vertex_row + 1) * vertex_row / 2
            };
            for vertex in 0..(vertex_row + 1) {
                let vertex_f = vertex as f32;
                let vertex_index = vertex_row_index * 3 + vertex * 3;
                verticies[0 + vertex_index] = vertex_row_start_x + vertex_f * t_width;
                verticies[1 + vertex_index] = vertex_row_y;
                verticies[2 + vertex_index] = 0.;

                if vertex_row < n_vertex_rows - 1 {
                    indicies[0 + vertex_index] = (vertex_row_index + vertex) as u32;
                    indicies[1 + vertex_index] =
                        (vertex_row_index + vertex_row + 1 + vertex) as u32;
                    indicies[2 + vertex_index] =
                        (vertex_row_index + vertex_row + 1 + vertex + 1) as u32;
                }
            }
        }

        (verticies, indicies)
    }

    pub fn generate_circle(n_perimeter_verticies: usize, rad: f32) -> (Vec<f32>, Vec<u32>) {
        let n_perimeter_verticies_f = n_perimeter_verticies as f32;

        // Three first represent center -> already 0;
        let mut verticies = vec![0.; (n_perimeter_verticies + 1) * 3];
        let mut indicies = vec![0; n_perimeter_verticies as usize * 3];

        for perimeter_vertex in 0..n_perimeter_verticies {
            let perimeter_vertex_f = perimeter_vertex as f32;
            let angle = (perimeter_vertex_f * 2. * pi::<f32>() / n_perimeter_verticies_f) as f64;

            let perimeter_index = perimeter_vertex * 3;

            verticies[3 + perimeter_index] = (angle.cos() as f32) * rad;
            verticies[4 + perimeter_index] = (angle.sin() as f32) * rad;
            verticies[5 + perimeter_index] = 0.;

            indicies[0 + perimeter_index] = 0;
            indicies[1 + perimeter_index] = (1 + perimeter_vertex) as u32;
            indicies[2 + perimeter_index] =
                (1 + (perimeter_vertex + 1) % n_perimeter_verticies) as u32;
        }

        (verticies, indicies)
    }

    pub fn generate_spiral(
        n_segments: usize,
        thickness: f32,
        windings: f32,
        spiral_outer_rad: f32,
        spiral_inner_rad: f32,
    ) -> (Vec<f32>, Vec<u32>) {
        let mut verticies = vec![0.; (n_segments * 2 + 2) * 3];
        let mut indicies = vec![0; (n_segments * 2) * 3];

        let n_segments_f = n_segments as f32;
        let spiral_delta_rad = spiral_outer_rad - spiral_inner_rad;
        for segment in 0..n_segments + 1 {
            let segment_f = segment as f32;
            let angle = segment_f * windings * 2. * pi::<f32>() / n_segments_f;

            let segment_outer_rad = spiral_inner_rad + spiral_delta_rad * segment_f / n_segments_f;
            let segment_inner_rad = segment_outer_rad - thickness;

            let segment_index = segment * 2 * 3;

            verticies[0 + segment_index] = (angle.cos() as f32) * segment_outer_rad;
            verticies[1 + segment_index] = (angle.sin() as f32) * segment_outer_rad;
            verticies[2 + segment_index] = 0.;

            verticies[3 + segment_index] = (angle.cos() as f32) * segment_inner_rad;
            verticies[4 + segment_index] = (angle.sin() as f32) * segment_inner_rad;
            verticies[5 + segment_index] = 0.;

            if segment < n_segments {
                indicies[0 + segment_index] = (2 + segment * 2) as u32;
                indicies[1 + segment_index] = (1 + segment * 2) as u32;
                indicies[2 + segment_index] = (0 + segment * 2) as u32;

                indicies[3 + segment_index] = (1 + segment * 2) as u32;
                indicies[4 + segment_index] = (2 + segment * 2) as u32;
                indicies[5 + segment_index] = (3 + segment * 2) as u32;
            }
        }

        (verticies, indicies)
    }

    pub fn generate_square(side_length: f32) -> (Vec<f32>, Vec<u32>) {
        let half_side_length = side_length / 2.;
        let mut verticies = vec![0.; 4 * 3];
        let mut indicies = vec![0; 2 * 3];

        // Anti-clockwise from top-right
        verticies[0] = half_side_length;
        verticies[1] = half_side_length;

        verticies[3] = -half_side_length;
        verticies[4] = half_side_length;

        verticies[6] = -half_side_length;
        verticies[7] = -half_side_length;

        verticies[9] = half_side_length;
        verticies[10] = -half_side_length;

        indicies.splice(0..3, vec![0, 1, 2]);
        indicies.splice(3..6, vec![2, 3, 0]);

        (verticies, indicies)
    }

    pub fn generate_sine(n_points: usize, width: f32, angle_rate: f32) -> (Vec<f32>, Vec<u32>) {
        let n_points_f = n_points as f32;
        let half_width = width / 2.;
        let mut verticies = vec![0.; (2 * n_points) * 3];
        let mut indicies = vec![0; (2 * n_points) * 3];

        let vertex_second_half_index = n_points * 3;

        let mut lowest_point = f32::MAX;
        for point in 0..n_points {
            let point_f = point as f32;
            let angle = (point_f * angle_rate) as f64;

            let y = angle.sin() as f32;
            let x = -half_width + (width * point_f / (n_points_f - 1.));

            let point_index = point * 3;
            verticies[0 + point_index] = x;
            verticies[1 + point_index] = y;
            // verticies[2 + point_index] = 0.;

            verticies[0 + point_index + vertex_second_half_index] = x;

            if point < n_points - 1 {
                indicies[0 + point_index * 2] = (1 + point) as u32;
                indicies[1 + point_index * 2] = (0 + point) as u32;
                indicies[2 + point_index * 2] = (0 + point + n_points) as u32;

                indicies[4 + point_index * 2] = (1 + point + n_points) as u32;
                indicies[3 + point_index * 2] = (0 + point + n_points) as u32;
                indicies[5 + point_index * 2] = (1 + point) as u32;
            }

            if y < lowest_point {
                lowest_point = y;
            }
        }

        verticies.splice(
            (n_points * 3)..,
            verticies[(n_points * 3)..]
                .iter()
                .enumerate()
                .map(|(i, &v)| if i % 3 == 1 { lowest_point } else { v })
                .collect::<Vec<f32>>(),
        );

        (verticies, indicies)
    }
}
