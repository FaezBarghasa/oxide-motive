use nalgebra::{SMatrix, SVector, Cholesky};

pub struct UnscentedKalmanFilter<const N: usize, const M: usize> {
    x: SVector<f32, N>,
    p: SMatrix<f32, N, N>,
    q: SMatrix<f32, N, N>,
    r: SMatrix<f32, M, M>,
    alpha: f32,
    beta: f32,
    kappa: f32,
}

impl<const N: usize, const M: usize> UnscentedKalmanFilter<N, M> {
    pub fn new(
        x0: SVector<f32, N>,
        p0: SMatrix<f32, N, N>,
        q: SMatrix<f32, N, N>,
        r: SMatrix<f32, M, M>,
        alpha: f32,
        beta: f32,
        kappa: f32,
    ) -> Self {
        Self {
            x: x0,
            p: p0,
            q,
            r,
            alpha,
            beta,
            kappa,
        }
    }

    pub fn predict<F>(&mut self, f: F, dt: f32)
    where
        F: Fn(SVector<f32, N>, f32) -> SVector<f32, N>,
    {
        let lambda = self.alpha.powi(2) * (N as f32 + self.kappa) - N as f32;
        let gamma = (N as f32 + lambda).sqrt();

        let mut sigma_points = SMatrix::<f32, N, { 2 * N + 1 }>::zeros();
        sigma_points.set_column(0, &self.x);

        let p_sqrt = self.p.cholesky().unwrap().l();
        for i in 0..N {
            let col = self.x + gamma * p_sqrt.column(i);
            sigma_points.set_column(i + 1, &col);
            let col = self.x - gamma * p_sqrt.column(i);
            sigma_points.set_column(i + 1 + N, &col);
        }

        let mut predicted_sigma_points = SMatrix::<f32, N, { 2 * N + 1 }>::zeros();
        for i in 0..(2 * N + 1) {
            let col = f(sigma_points.column(i).into(), dt);
            predicted_sigma_points.set_column(i, &col);
        }

        let wm0 = lambda / (N as f32 + lambda);
        let wc0 = wm0 + (1.0 - self.alpha.powi(2) + self.beta);
        let wmi = 1.0 / (2.0 * (N as f32 + lambda));

        let mut x_pred = wm0 * predicted_sigma_points.column(0);
        for i in 1..(2 * N + 1) {
            x_pred += wmi * predicted_sigma_points.column(i);
        }

        let mut p_pred = wc0 * (predicted_sigma_points.column(0) - x_pred) * (predicted_sigma_points.column(0) - x_pred).transpose();
        for i in 1..(2 * N + 1) {
            p_pred += wmi * (predicted_sigma_points.column(i) - x_pred) * (predicted_sigma_points.column(i) - x_pred).transpose();
        }
        p_pred += self.q;

        self.x = x_pred;
        self.p = p_pred;
    }

    pub fn update<H>(&mut self, z: SVector<f32, M>, h: H)
    where
        H: Fn(SVector<f32, N>) -> SVector<f32, M>,
    {
        let lambda = self.alpha.powi(2) * (N as f32 + self.kappa) - N as f32;
        let gamma = (N as f32 + lambda).sqrt();

        let mut sigma_points = SMatrix::<f32, N, { 2 * N + 1 }>::zeros();
        sigma_points.set_column(0, &self.x);

        let p_sqrt = self.p.cholesky().unwrap().l();
        for i in 0..N {
            let col = self.x + gamma * p_sqrt.column(i);
            sigma_points.set_column(i + 1, &col);
            let col = self.x - gamma * p_sqrt.column(i);
            sigma_points.set_column(i + 1 + N, &col);
        }

        let mut z_sigma_points = SMatrix::<f32, M, { 2 * N + 1 }>::zeros();
        for i in 0..(2 * N + 1) {
            let col = h(sigma_points.column(i).into());
            z_sigma_points.set_column(i, &col);
        }

        let wm0 = lambda / (N as f32 + lambda);
        let wc0 = wm0 + (1.0 - self.alpha.powi(2) + self.beta);
        let wmi = 1.0 / (2.0 * (N as f32 + lambda));

        let mut z_pred = wm0 * z_sigma_points.column(0);
        for i in 1..(2 * N + 1) {
            z_pred += wmi * z_sigma_points.column(i);
        }

        let mut s = wc0 * (z_sigma_points.column(0) - z_pred) * (z_sigma_points.column(0) - z_pred).transpose();
        for i in 1..(2 * N + 1) {
            s += wmi * (z_sigma_points.column(i) - z_pred) * (z_sigma_points.column(i) - z_pred).transpose();
        }
        s += self.r;

        let mut t = wc0 * (sigma_points.column(0) - self.x) * (z_sigma_points.column(0) - z_pred).transpose();
        for i in 1..(2 * N + 1) {
            t += wmi * (sigma_points.column(i) - self.x) * (z_sigma_points.column(i) - z_pred).transpose();
        }

        let k = t * s.try_inverse().unwrap();
        self.x += k * (z - z_pred);
        self.p -= k * s * k.transpose();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector1;

    #[test]
    fn test_ukf() {
        let mut ukf = UnscentedKalmanFilter::<1, 1>::new(
            SVector::new(0.0),
            SMatrix::identity(),
            SMatrix::identity() * 0.01,
            SMatrix::identity() * 0.1,
            1e-3,
            2.0,
            0.0,
        );

        let f = |x: SVector<f32, 1>, dt: f32| -> SVector<f32, 1> {
            SVector::new(x[0] + dt)
        };

        let h = |x: SVector<f32, 1>| -> SVector<f32, 1> {
            x
        };

        for i in 0..10 {
            let z = SVector::new(i as f32);
            ukf.predict(f, 1.0);
            ukf.update(z, h);
        }

        assert!((ukf.x[0] - 10.0).abs() < 1.0);
    }
}
