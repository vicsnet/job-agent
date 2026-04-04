pub fn check_similarity(a: &[f32], b: &[f32]) -> f32 {

    let dot:f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a:f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b:f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
    
}