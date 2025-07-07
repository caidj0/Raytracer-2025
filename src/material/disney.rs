use std::f64::consts::PI;

use crate::{
    material::{Material, ScatterRecord, ScatterType},
    pdf::DisneyPDF,
    utils::{
        color::Color,
        fresnel::{dielectric, schlick, schlick_f64, schlick_r0_from_relative_ior, schlick_weight},
        lerp,
        onb::OrthonormalBasis,
        random::Random,
        vec3::{UnitVec3, Vec3},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Disney {
    pub base_color: Color,
    pub roughness: f64,
    pub anisotropic: f64,

    pub sheen: f64,
    pub sheen_tint: f64,
    pub clearcoat: f64,
    pub clearcoat_alpha: f64,
    pub relative_ior: f64,
    pub specular_tint: f64,
    pub metallic: f64,
    pub ior: f64,
    pub flatness: f64,
}

impl Material for Disney {
    fn scatter(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
    ) -> Option<super::ScatterRecord> {
        let v_in = UnitVec3::from_vec3(-r_in.direction()).unwrap();
        let disney_pdf = Box::new(DisneyPDF::new(self, &rec.normal, &v_in));

        let attenuation = self.base_color;

        Some(ScatterRecord {
            attenuation: attenuation,
            scatter_type: ScatterType::PDF(disney_pdf),
        })
    }

    fn scattering_pdf(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
        scattered: &crate::utils::ray::Ray,
    ) -> f64 {
        let uvw = OrthonormalBasis::new(&rec.normal);
        let v_in = UnitVec3::from_vec3_raw(
            uvw.world_to_onb(UnitVec3::from_vec3(-r_in.direction()).unwrap().into_inner()),
        );
        let v_out = UnitVec3::from_vec3_raw(
            uvw.world_to_onb(
                UnitVec3::from_vec3(*scattered.direction())
                    .unwrap()
                    .into_inner(),
            ),
        );

        if v_in.cos_theta() <= 0.0 || v_out.cos_theta() <= 0.0 {
            return 0.0;
        }

        // 计算半向量
        let v_half = UnitVec3::from_vec3(v_in.as_inner() + v_out.as_inner()).unwrap();

        // 使用现有的评估函数计算BRDF值
        let (brdf_spec, _, _) = self.evaluate_brdf(&v_out, &v_half, &v_in);
        let diffuse_brdf = self.evaluate_disney_diffuse(&v_out, &v_half, &v_in, false);
        let sheen_brdf = self.evaluate_sheen(&v_out, &v_half, &v_in);
        let (clearcoat_brdf, _, _) = self.evaluate_clearcoat(&v_out, &v_half, &v_in);

        // 组合所有BRDF分量
        let total_brdf = brdf_spec
            + self.base_color * diffuse_brdf * (1.0 - self.metallic)
            + sheen_brdf
            + Color::new(clearcoat_brdf, clearcoat_brdf, clearcoat_brdf);

        // 返回BRDF的亮度作为PDF权重
        total_brdf.luminance() * v_out.cos_theta()
    }
}

impl Disney {
    fn evaluate_brdf(
        &self,
        v_out: &UnitVec3,
        v_half: &UnitVec3,
        v_in: &UnitVec3,
    ) -> (Color, f64, f64) {
        let dot_nl = v_in.cos_theta();
        let dot_nv = v_out.cos_theta();
        if dot_nl <= 0.0 || dot_nv <= 0.0 {
            return (Color::BLACK, 0.0, 0.0);
        }

        let (ax, ay) = calculate_anisotropic_params(self.roughness, self.anisotropic);

        let d = ggx_anisotropic_d(v_half, ax, ay);
        let gl = anisotropic_separable_smith_ggxg1(v_in, v_half, ax, ay);
        let gv = anisotropic_separable_smith_ggxg1(v_out, v_half, ax, ay);

        let f = self.disney_fresnel(v_out, v_half, v_in);

        let (forward_pdf, reverse_pdf) = ggx_vndf_anisotropic_pdf(v_in, v_half, v_out, ax, ay);

        let forward_pdf = forward_pdf * (1.0 / (4.0 * v_out.dot(&v_half).abs()));
        let reverse_pdf = reverse_pdf * (1.0 / (4.0 * v_in.dot(&v_half).abs()));

        let value = d * gl * gv * f / (4.0 * dot_nl * dot_nv);
        (value, forward_pdf, reverse_pdf)
    }

    fn evaluate_sheen(&self, _v_out: &UnitVec3, v_half: &UnitVec3, v_in: &UnitVec3) -> Color {
        if self.sheen <= 0.0 {
            return Color::BLACK;
        }

        let dot_hl = v_half.dot(v_in);
        let tint = calculate_tint(self.base_color);
        self.sheen * lerp(Vec3::new(1.0, 1.0, 1.0), tint, self.sheen_tint) * schlick_weight(dot_hl)
    }

    fn evaluate_clearcoat(
        &self,
        v_out: &UnitVec3,
        v_half: &UnitVec3,
        v_in: &UnitVec3,
    ) -> (f64, f64, f64) {
        if self.clearcoat <= 0.0 {
            return (0.0, 0.0, 0.0);
        }

        // N, H, L, V 分别代表 法线，半向量，入射向量，出射向量
        let dot_nh = v_half.y();
        let dot_nl = v_in.y();
        let dot_nv = v_out.y();
        let dot_hl = v_half.dot(&v_in);

        let d = gtr1(dot_nh, lerp(0.1, 0.001, self.clearcoat_alpha));
        let f = schlick_f64(0.04, dot_hl);
        let gl = separable_smith_ggxg1(v_in, 0.25);
        let gv = separable_smith_ggxg1(v_out, 0.25);

        let value = 0.25 * self.clearcoat * d * f * gl * gv;
        let forward_pdf = d / (4.0 * dot_nl.abs());
        let reverse_pdf = d / (4.0 * dot_nv.abs());

        (value, forward_pdf, reverse_pdf)
    }

    fn disney_fresnel(&self, v_out: &UnitVec3, v_half: &UnitVec3, v_in: &UnitVec3) -> Color {
        let dot_hv = v_half.dot(&v_out);

        let tint = calculate_tint(self.base_color);

        let r0 = schlick_r0_from_relative_ior(self.relative_ior)
            * lerp(Vec3::new(1.0, 1.0, 1.0), tint, self.specular_tint);
        let r0 = lerp(r0, self.base_color, self.metallic);

        let dielectric_fresnel = dielectric(dot_hv, 1.0, self.ior);
        let metallic_fresnel = schlick(r0, v_in.dot(&v_half));

        lerp(
            Vec3::new(dielectric_fresnel, dielectric_fresnel, dielectric_fresnel),
            metallic_fresnel,
            self.metallic,
        )
    }

    fn evaluate_disney_spec_transmission(
        &self,
        v_out: &UnitVec3,
        v_half: &UnitVec3,
        v_in: &UnitVec3,
        ax: f64,
        ay: f64,
        thin: bool,
    ) -> Color {
        let relative_ior = self.relative_ior;
        let n2 = relative_ior * relative_ior;

        let abs_dot_nl = v_in.cos_theta().abs();
        let abs_dot_nv = v_out.cos_theta().abs();
        let dot_hl = v_half.dot(&v_in);
        let dot_hv = v_half.dot(&v_out);
        let abs_dot_hl = dot_hl.abs();
        let abs_dot_hv = dot_hv.abs();

        let d = ggx_anisotropic_d(v_half, ax, ay);
        let gl = anisotropic_separable_smith_ggxg1(v_in, v_half, ax, ay);
        let gv = anisotropic_separable_smith_ggxg1(v_out, v_half, ax, ay);

        let f = dielectric(dot_hv, 1.0, 1.0 / relative_ior);

        let color = if thin {
            self.base_color.sqrt()
        } else {
            self.base_color
        };

        let c = (abs_dot_hl * abs_dot_hv) / (abs_dot_nl * abs_dot_nv);
        let t = n2 / (dot_hl + relative_ior * dot_hv).powi(2);
        c * t * (1.0 - f) * gl * gv * d * color
    }

    fn evaluate_disney_diffuse(
        &self,
        v_out: &UnitVec3,
        v_half: &UnitVec3,
        v_in: &UnitVec3,
        thin: bool,
    ) -> f64 {
        let abs_dot_nl = v_in.cos_theta().abs();
        let abs_dot_nv = v_out.cos_theta().abs();

        let fl = schlick_weight(abs_dot_nl);
        let fv = schlick_weight(abs_dot_nv);

        let hanrahan_krueger = if thin && self.flatness > 0.0 {
            let roughness = self.roughness * self.roughness;

            let dot_hl = v_half.dot(&v_in);
            let fss90 = dot_hl * dot_hl * roughness;
            let fss = lerp(1.0, fss90, fl) * lerp(1.0, fss90, fv);

            1.25 * (fss * (1.0 / (abs_dot_nl + abs_dot_nv) - 0.5) + 0.5)
        } else {
            0.0
        };

        let lambert = 1.0;
        let retro = self.evaluate_disney_retro_diffuse(v_out, v_half, v_in);
        let subsurface_approx = lerp(
            lambert,
            hanrahan_krueger,
            if thin { self.flatness } else { 0.0 },
        );

        1.0 / PI * (retro + subsurface_approx * (1.0 - 0.5 * fl) * (1.0 - 0.5 * fv))
    }

    fn evaluate_disney_retro_diffuse(
        &self,
        v_out: &UnitVec3,
        _v_half: &UnitVec3,
        v_in: &UnitVec3,
    ) -> f64 {
        let abs_dot_nl = v_in.cos_theta().abs();
        let abs_dot_nv = v_out.cos_theta().abs();

        let roughness = self.roughness * self.roughness;

        let rr = 0.5 + 2.0 * abs_dot_nl * abs_dot_nl * roughness;
        let fl = schlick_weight(abs_dot_nl);
        let fv = schlick_weight(abs_dot_nv);

        rr * (fl + fv + fl * fv * (rr - 1.0))
    }

    // 采样Disney BSDF - 使用多重重要性采样
    pub fn sample_disney_bsdf(&self, v_in: &UnitVec3) -> (UnitVec3, f64) {
        // 计算各个lobe的采样权重
        let specular_weight = self.calculate_specular_weight();
        let diffuse_weight = self.calculate_diffuse_weight();
        let clearcoat_weight = self.calculate_clearcoat_weight();
        let sheen_weight = self.calculate_sheen_weight();

        let total_weight = specular_weight + diffuse_weight + clearcoat_weight + sheen_weight;

        if total_weight <= 0.0 {
            return self.sample_cosine_weighted();
        }

        let r = Random::f64() * total_weight;
        let mut accumulated_weight = 0.0;

        // 根据权重选择采样的lobe
        accumulated_weight += specular_weight;
        if r < accumulated_weight {
            return self.sample_specular_lobe(v_in);
        }

        accumulated_weight += diffuse_weight;
        if r < accumulated_weight {
            return self.sample_diffuse_lobe();
        }

        accumulated_weight += clearcoat_weight;
        if r < accumulated_weight {
            return self.sample_clearcoat_lobe();
        }

        // 默认采样sheen（简化为余弦采样）
        self.sample_cosine_weighted()
    }

    // 计算总PDF - 利用现有的评估函数
    pub fn calculate_total_pdf(&self, v_out: &UnitVec3, v_in: &UnitVec3) -> f64 {
        let v_half = UnitVec3::from_vec3(v_in.as_inner() + v_out.as_inner()).unwrap();

        // 使用现有函数计算各个lobe的PDF
        let (ax, ay) = calculate_anisotropic_params(self.roughness, self.anisotropic);
        let (spec_pdf, _) = ggx_vndf_anisotropic_pdf(v_in, &v_half, v_out, ax, ay);
        let spec_pdf = spec_pdf * (1.0 / (4.0 * v_out.dot(&v_half).abs()));

        let diffuse_pdf = v_out.cos_theta() / PI;

        let (_, clearcoat_pdf, _) = self.evaluate_clearcoat(v_out, &v_half, v_in);

        // 计算权重
        let specular_weight = self.calculate_specular_weight();
        let diffuse_weight = self.calculate_diffuse_weight();
        let clearcoat_weight = self.calculate_clearcoat_weight();

        let total_weight = specular_weight + diffuse_weight + clearcoat_weight;

        if total_weight <= 0.0 {
            return diffuse_pdf;
        }

        // 混合PDF
        (specular_weight * spec_pdf
            + diffuse_weight * diffuse_pdf
            + clearcoat_weight * clearcoat_pdf)
            / total_weight
    }

    // 计算各个lobe的采样权重
    fn calculate_specular_weight(&self) -> f64 {
        self.metallic + (1.0 - self.metallic) * self.specular_tint
    }

    fn calculate_diffuse_weight(&self) -> f64 {
        (1.0 - self.metallic) * (1.0 - self.specular_tint)
    }

    fn calculate_clearcoat_weight(&self) -> f64 {
        self.clearcoat * 0.25
    }

    fn calculate_sheen_weight(&self) -> f64 {
        self.sheen * 0.1
    }

    // 采样镜面反射lobe
    fn sample_specular_lobe(&self, v_in: &UnitVec3) -> (UnitVec3, f64) {
        let (ax, ay) = calculate_anisotropic_params(self.roughness, self.anisotropic);

        // 使用GGX VNDF采样半向量
        let v_half = self.sample_ggx_vndf(v_in, ax, ay);

        // 计算反射方向
        let v_out = self.reflect_vector(v_in, &v_half);

        if v_out.cos_theta() <= 0.0 {
            return (UnitVec3::from_vec3_raw(Vec3::new(0.0, 1.0, 0.0)), 0.0);
        }

        let pdf = self.calculate_total_pdf(&v_out, v_in);
        (v_out, pdf)
    }

    // 采样漫反射lobe
    fn sample_diffuse_lobe(&self) -> (UnitVec3, f64) {
        self.sample_cosine_weighted()
    }

    // 采样清漆lobe
    fn sample_clearcoat_lobe(&self) -> (UnitVec3, f64) {
        if self.clearcoat <= 0.0 {
            return self.sample_cosine_weighted();
        }

        let alpha = lerp(0.1, 0.001, self.clearcoat_alpha);
        let v_half = self.sample_gtr1(alpha);

        // 简化：假设视线方向垂直向上
        let v_view = UnitVec3::from_vec3_raw(Vec3::new(0.0, 1.0, 0.0));
        let v_out = self.reflect_vector(&v_view, &v_half);

        if v_out.cos_theta() <= 0.0 {
            return self.sample_cosine_weighted();
        }

        let pdf = self.calculate_total_pdf(&v_out, &v_view);
        (v_out, pdf)
    }

    // 余弦加权采样
    fn sample_cosine_weighted(&self) -> (UnitVec3, f64) {
        let r1 = Random::f64();
        let r2 = Random::f64();

        let cos_theta = r1.sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * PI * r2;

        let v_out = UnitVec3::from_vec3_raw(Vec3::new(
            sin_theta * phi.cos(),
            cos_theta,
            sin_theta * phi.sin(),
        ));

        let pdf = cos_theta / PI;
        (v_out, pdf)
    }

    // 采样GGX VNDF（简化版本）
    fn sample_ggx_vndf(&self, _v_in: &UnitVec3, ax: f64, ay: f64) -> UnitVec3 {
        let r1 = Random::f64();
        let r2 = Random::f64();

        // 简化的GGX采样
        let alpha = (ax + ay) * 0.5; // 简化为各向同性
        let alpha2 = alpha * alpha;

        let cos_theta = ((1.0 - r1) / (r1 * (alpha2 - 1.0) + 1.0)).sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * PI * r2;

        UnitVec3::from_vec3_raw(Vec3::new(
            sin_theta * phi.cos(),
            cos_theta,
            sin_theta * phi.sin(),
        ))
    }

    // 反射向量计算
    fn reflect_vector(&self, v_in: &UnitVec3, v_half: &UnitVec3) -> UnitVec3 {
        let dot = v_in.dot(v_half);
        UnitVec3::from_vec3(2.0 * dot * v_half.as_inner() - v_in.as_inner()).unwrap()
    }

    // 采样GTR1分布
    fn sample_gtr1(&self, alpha: f64) -> UnitVec3 {
        let r1 = Random::f64();
        let r2 = Random::f64();

        let a2 = alpha * alpha;
        let cos_theta = if a2 < 1.0 {
            ((1.0 - r1.powf(2.0 / (1.0 + alpha)))
                / (1.0 - r1.powf(2.0 / (1.0 + alpha)) + r1.powf(2.0 / (1.0 + alpha))))
            .sqrt()
        } else {
            r1.sqrt()
        };

        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * PI * r2;

        UnitVec3::from_vec3_raw(Vec3::new(
            sin_theta * phi.cos(),
            cos_theta,
            sin_theta * phi.sin(),
        ))
    }
}

fn calculate_tint(base_color: Color) -> Color {
    let luminance = Color::new(0.3, 0.6, 1.0).dot(&base_color);

    if luminance > 0.0 {
        base_color * (1.0 / luminance)
    } else {
        Color::WHITE
    }
}

fn gtr1(dot_hl: f64, a: f64) -> f64 {
    if a >= 1.0 {
        return 1.0 / PI;
    }

    let a2 = a * a;

    (a2 - 1.0) / (PI * a2.ln() * (1.0 + (a2 - 1.0) * dot_hl * dot_hl))
}

fn separable_smith_ggxg1(w: &UnitVec3, a: f64) -> f64 {
    let a2 = a * a;
    let dot_nv = w.y();

    2.0 / (1.0 + (a2 + (1.0 - a2) * dot_nv * dot_nv).sqrt())
}

fn ggx_anisotropic_d(v_half: &UnitVec3, ax: f64, ay: f64) -> f64 {
    let dot_hx2 = v_half.x().powi(2);
    let dot_hy2 = v_half.z().powi(2);
    let cos_theta2 = v_half.y().powi(2);
    let ax2 = ax * ax;
    let ay2 = ay * ay;

    1.0 / (PI * ax * ay * (dot_hx2 / ax2 + dot_hy2 / ay2 + cos_theta2).powi(2))
}

fn anisotropic_separable_smith_ggxg1(w: &UnitVec3, v_half: &UnitVec3, ax: f64, ay: f64) -> f64 {
    let dot_hw = w.dot(&v_half);
    if dot_hw <= 0.0 {
        return 0.0;
    }

    let abs_tan_theta = w.tan_theta().abs();
    assert!(!abs_tan_theta.is_nan());
    if abs_tan_theta.is_infinite() {
        return 0.0;
    }

    let a = (w.cos_phi() * ax * ax + w.sin_phi2() * ay * ay).sqrt();
    let a2_tan_theta2 = (a * abs_tan_theta).powi(2);

    let lambda = 0.5 * (-1.0 + (1.0 + a2_tan_theta2).sqrt());

    1.0 / (1.0 + lambda)
}

fn calculate_anisotropic_params(roughness: f64, anisotropic: f64) -> (f64, f64) {
    let aspect = (1.0 - 0.9 * anisotropic).sqrt();
    let roughness2 = roughness * roughness;
    let ax = f64::max(0.001, roughness2 / aspect);
    let ay = f64::max(0.001, roughness2 * aspect);
    (ax, ay)
}

fn ggx_vndf_anisotropic_pdf(
    v_in: &UnitVec3,
    v_half: &UnitVec3,
    v_out: &UnitVec3,
    ax: f64,
    ay: f64,
) -> (f64, f64) {
    let d = ggx_anisotropic_d(v_half, ax, ay);

    let abs_dot_nl = v_in.cos_theta().abs();
    let abs_dot_hl = v_half.dot(v_in).abs();
    let g1v = anisotropic_separable_smith_ggxg1(v_out, v_half, ax, ay);
    let forward_pdf_weight = g1v * abs_dot_hl * d / abs_dot_nl;

    let abs_dot_nv = v_out.cos_theta().abs();
    let abs_dot_hv = v_half.dot(v_out).abs();
    let g1l = anisotropic_separable_smith_ggxg1(v_in, v_half, ax, ay);
    let reverse_pdf_weight = g1l * abs_dot_hv * d / abs_dot_nv;

    (forward_pdf_weight, reverse_pdf_weight)
}

fn thin_transmission_roughness(ior: f64, roughness: f64) -> f64 {
    ((0.65 * ior - 0.35) * roughness).clamp(0.0, 1.0)
}
