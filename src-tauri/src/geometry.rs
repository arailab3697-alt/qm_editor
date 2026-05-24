use crate::domain::Element;

pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

pub fn dihedral_degrees(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> Option<f64> {
    let b0 = sub(b, a);
    let b1 = sub(c, b);
    let b2 = sub(d, c);
    let n1 = normalize(cross(b0, b1))?;
    let n2 = normalize(cross(b1, b2))?;
    let m1 = cross(n1, normalize(b1)?);
    Some(dot(m1, n2).atan2(dot(n1, n2)).to_degrees())
}

pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(vector: [f64; 3], scalar: f64) -> [f64; 3] {
    [vector[0] * scalar, vector[1] * scalar, vector[2] * scalar]
}

pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn length(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

pub fn normalize(vector: [f64; 3]) -> Option<[f64; 3]> {
    let vector_length = length(vector);
    if vector_length <= f64::EPSILON {
        None
    } else {
        Some(scale(vector, 1.0 / vector_length))
    }
}

pub fn rotate(vector: [f64; 3], axis: [f64; 3], radians: f64) -> [f64; 3] {
    let cos = radians.cos();
    let sin = radians.sin();
    add(
        add(scale(vector, cos), scale(cross(axis, vector), sin)),
        scale(axis, dot(axis, vector) * (1.0 - cos)),
    )
}

pub fn perpendicular(vector: [f64; 3]) -> [f64; 3] {
    let candidate = if vector[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    normalize(cross(vector, candidate)).unwrap_or([0.0, 0.0, 1.0])
}

pub fn covalent_radius(element: Element) -> f64 {
    match element {
        Element::H => 0.31,
        Element::C => 0.76,
        Element::N => 0.71,
        Element::O => 0.66,
        Element::F => 0.57,
        Element::P => 1.07,
        Element::S => 1.05,
        Element::Cl => 1.02,
        Element::Br => 1.20,
        Element::I => 1.39,
        _ => 0.75,
    }
}

pub fn rotation_from_to(from: [f64; 3], to: [f64; 3]) -> [[f64; 3]; 3] {
    let axis = cross(from, to);
    let s = length(axis);
    let c = dot(from, to);

    if s < 1e-6 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }

    let vx = [
        [0.0, -axis[2], axis[1]],
        [axis[2], 0.0, -axis[0]],
        [-axis[1], axis[0], 0.0],
    ];
    let vx2 = mat_mul(vx, vx);
    add_mat(
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        add_mat(vx, scale_mat(vx2, (1.0 - c) / s.powi(2))),
    )
}

pub fn rotate_vec(rotation: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        dot(rotation[0], v),
        dot(rotation[1], v),
        dot(rotation[2], v),
    ]
}

pub fn mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut res = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                res[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    res
}

pub fn add_mat(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut res = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            res[i][j] = a[i][j] + b[i][j];
        }
    }
    res
}

pub fn scale_mat(a: [[f64; 3]; 3], s: f64) -> [[f64; 3]; 3] {
    let mut res = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            res[i][j] = a[i][j] * s;
        }
    }
    res
}
