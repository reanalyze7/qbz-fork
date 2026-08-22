//! Pure numeric k-means clustering on RGB values, with no dependency on
//! `image`/`Path`.

/// A cluster result: centroid + pixel count.
#[derive(Debug, Clone)]
pub(super) struct Cluster {
    pub(super) centroid: [f64; 3],
    pub(super) count: usize,
}

/// Simple k-means clustering on RGB values.
pub(super) fn kmeans(pixels: &[[f64; 3]], k: usize, max_iters: usize) -> Vec<Cluster> {
    let n = pixels.len();
    if n == 0 || k == 0 {
        return Vec::new();
    }
    let k = k.min(n);

    // Initialize centroids by evenly sampling from the pixel list.
    let mut centroids: Vec<[f64; 3]> = Vec::with_capacity(k);
    let step = n / k;
    for i in 0..k {
        centroids.push(pixels[i * step]);
    }

    let mut assignments = vec![0usize; n];

    for _ in 0..max_iters {
        let mut changed = false;

        // Assignment step.
        for (idx, pixel) in pixels.iter().enumerate() {
            let mut best_cluster = 0;
            let mut best_dist = f64::MAX;
            for (ci, centroid) in centroids.iter().enumerate() {
                let dist = rgb_dist_sq(pixel, centroid);
                if dist < best_dist {
                    best_dist = dist;
                    best_cluster = ci;
                }
            }
            if assignments[idx] != best_cluster {
                assignments[idx] = best_cluster;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Update step.
        let mut sums = vec![[0.0f64; 3]; k];
        let mut counts = vec![0usize; k];
        for (idx, pixel) in pixels.iter().enumerate() {
            let ci = assignments[idx];
            sums[ci][0] += pixel[0];
            sums[ci][1] += pixel[1];
            sums[ci][2] += pixel[2];
            counts[ci] += 1;
        }

        for ci in 0..k {
            if counts[ci] > 0 {
                centroids[ci][0] = sums[ci][0] / counts[ci] as f64;
                centroids[ci][1] = sums[ci][1] / counts[ci] as f64;
                centroids[ci][2] = sums[ci][2] / counts[ci] as f64;
            }
        }
    }

    let mut counts = vec![0usize; k];
    for &a in &assignments {
        counts[a] += 1;
    }

    centroids
        .into_iter()
        .zip(counts)
        .filter(|(_, count)| *count > 0)
        .map(|(centroid, count)| Cluster { centroid, count })
        .collect()
}

fn rgb_dist_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    dr * dr + dg * dg + db * db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kmeans_basic_two_clusters() {
        let pixels: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0],
            [10.0, 10.0, 10.0],
            [5.0, 5.0, 5.0],
            [250.0, 250.0, 250.0],
            [240.0, 240.0, 240.0],
            [245.0, 245.0, 245.0],
        ];
        let clusters = kmeans(&pixels, 2, 20);
        assert_eq!(clusters.len(), 2);
    }
}
