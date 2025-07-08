use std::f64::consts::PI;

use crate::{
    material::{Material, ScatterRecord, ScatterType},
    pdf::PDF,
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
    pub clearcoat_gloss: f64,
    pub relative_ior: f64,
    pub specular_tint: f64,
    pub metallic: f64,
    pub ior: f64,
    pub flatness: f64,
    pub spec_trans: f64,
    pub diff_trans: f64,
}

impl Material for Disney {
    fn scatter(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
    ) -> Option<super::ScatterRecord> {
        let v_out = UnitVec3::from_vec3(-r_in.direction()).unwrap();

        let disney_pdf = Box::new(DisneyPDF::new(self, &rec.normal, &v_out, false));

        Some(ScatterRecord {
            attenuation: Color::new(10000.0, 10000.0, 10000.0), // This value is not used, the real attenuation is calculated in the PDF value
            scatter_type: ScatterType::PDF(disney_pdf),
        })
    }

    fn scattering_pdf(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
        scattered: &crate::utils::ray::Ray,
    ) -> f64 {
        let v_out = UnitVec3::from_vec3(-r_in.direction()).unwrap();
        let disney_pdf = DisneyPDF::new(self, &rec.normal, &v_out, false);

        let (_, pdf) = disney_pdf.value(scattered.direction());
        pdf
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
        let dot_hl = v_half.dot(&v_in);

        let d = gtr1(dot_nh, lerp(0.1, 0.001, self.clearcoat_gloss));
        let f = schlick_f64(0.04, dot_hl);
        let gl = separable_smith_ggxg1(v_in, 0.25);
        let gv = separable_smith_ggxg1(v_out, 0.25);

        let value = 0.25 * self.clearcoat * d * f * gl * gv;
        let forward_pdf = d / (4.0 * v_out.dot(&v_half).abs());
        let reverse_pdf = d / (4.0 * v_in.dot(&v_half).abs());

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

    pub fn evaluate_disney(
        &self,
        v_out: &UnitVec3,
        v_in: &UnitVec3,
        thin: bool,
    ) -> (Color, f64, f64) {
        let v_half = UnitVec3::from_vec3(v_in.as_inner() + v_out.as_inner())
            .expect("The half veator can't be normalized!");

        let dot_nv = v_out.cos_theta();
        let dot_nl = v_in.cos_theta();

        let mut reflectance = Color::ZERO;
        let mut forward_pdf = 0.0;
        let mut reverse_pdf = 0.0;

        let (p_brdf, p_diffuse, p_clearcoat, p_spec_trans) = self.calculate_lobe_pdfs();

        let metallic = self.metallic;
        let spec_trans = self.spec_trans;

        let diffuse_weight = (1.0 - metallic) * (1.0 - spec_trans);
        let trans_weight = (1.0 - metallic) * spec_trans;

        let upper_hemisphere = dot_nl > 0.0 && dot_nv > 0.0;

        if upper_hemisphere && self.clearcoat > 0.0 {
            let (clearcoat, forward_clearcoat_pdf_w, reverse_clearcoat_pdf_w) =
                self.evaluate_clearcoat(v_out, &v_half, v_in);

            reflectance += Vec3::new(clearcoat, clearcoat, clearcoat);
            forward_pdf += p_clearcoat * forward_clearcoat_pdf_w;
            reverse_pdf += p_clearcoat * reverse_clearcoat_pdf_w;
        };

        if diffuse_weight > 0.0 {
            let forward_diffuse_pdf_w = v_in.cos_theta().abs();
            let reverse_diffuse_pdf_w = v_out.cos_theta().abs();
            let diffuse = self.evaluate_disney_diffuse(v_out, &v_half, v_in, thin);

            let sheen = self.evaluate_sheen(v_out, &v_half, v_in);

            reflectance += diffuse_weight * (diffuse * self.base_color + sheen);
            forward_pdf += p_diffuse * forward_diffuse_pdf_w;
            reverse_pdf += p_diffuse * reverse_diffuse_pdf_w;
        };

        if trans_weight > 0.0 {
            let rscaled = if thin {
                thin_transmission_roughness(self.ior, self.roughness)
            } else {
                self.roughness
            };

            let (tax, tay) = calculate_anisotropic_params(rscaled, self.anisotropic);

            let transmission =
                self.evaluate_disney_spec_transmission(v_out, &v_half, v_in, tax, tay, thin);
            reflectance += trans_weight * transmission;

            let (forward_transmissive_pdf_w, reverse_transmissive_pdf_w) =
                ggx_vndf_anisotropic_pdf(v_in, &v_half, v_out, tax, tay);

            let dot_lh = v_half.dot(&v_in);
            let dot_vh = v_half.dot(&v_out);

            forward_pdf += p_spec_trans * forward_transmissive_pdf_w
                / (dot_lh + self.relative_ior * dot_vh).powi(2);
            reverse_pdf += p_spec_trans * reverse_transmissive_pdf_w
                / (dot_vh + self.relative_ior * dot_lh).powi(2);
        }

        if upper_hemisphere {
            let (specular, forward_metallic_pdf_w, reverse_metallic_pdf_w) =
                self.evaluate_brdf(v_out, &v_half, v_in);

            reflectance += specular;
            forward_pdf += p_brdf * forward_metallic_pdf_w / (4.0 * v_out.dot(&v_half).abs());
            reverse_pdf += p_brdf * reverse_metallic_pdf_w / (4.0 * v_in.dot(&v_half).abs());
        }

        reflectance = reflectance * dot_nl.abs();

        (reflectance, forward_pdf, reverse_pdf)
    }

    fn calculate_lobe_pdfs(&self) -> (f64, f64, f64, f64) {
        let metallic_brdf = self.metallic;
        let specular_bsdf = (1.0 - self.metallic) * self.spec_trans;
        let dielectric_brdf = (1.0 - self.spec_trans) * (1.0 - self.metallic);

        let specular_weight = metallic_brdf + dielectric_brdf;
        let transmission_weight = specular_bsdf;
        let diffuse_weight = dielectric_brdf;
        let clearcoat_weight = 1.0 * self.clearcoat.clamp(0.0, 1.0);

        let norm =
            1.0 / (specular_weight + transmission_weight + diffuse_weight + clearcoat_weight);

        let p_specular = specular_weight * norm;
        let p_spec_trans = transmission_weight * norm;
        let p_diffuse = diffuse_weight * norm;
        let p_clearcoat = clearcoat_weight * norm;

        (p_specular, p_diffuse, p_clearcoat, p_spec_trans)
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

    let a = (w.cos_phi2() * ax * ax + w.sin_phi2() * ay * ay).sqrt();
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

pub struct DisneyPDF<'a> {
    material: &'a Disney,
    uvw: OrthonormalBasis,
    v_out: UnitVec3,
    thin: bool,
}

impl<'a> DisneyPDF<'a> {
    pub fn new(material: &'a Disney, normal: &UnitVec3, v_out: &UnitVec3, thin: bool) -> Self {
        let uvw = OrthonormalBasis::new(normal);
        let v_out = UnitVec3::from_vec3_raw(uvw.world_to_onb(v_out.into_inner()));

        Self {
            material,
            uvw,
            v_out,
            thin,
        }
    }

    fn sample_disney_brdf(&self) -> Option<UnitVec3> {
        let v_out = &self.v_out;

        let (ax, ay) =
            calculate_anisotropic_params(self.material.roughness, self.material.anisotropic);

        let r0 = Random::f64();
        let r1 = Random::f64();
        let v_half = sample_ggx_vndf_anisotropic(&v_out, ax, ay, r0, r1);

        let v_in = UnitVec3::from_vec3(v_out.reflect2(&v_half)).unwrap();

        if v_in.cos_theta() <= 0.0 {
            None
        } else {
            Some(UnitVec3::from_vec3(self.uvw.onb_to_world(v_in.into_inner())).unwrap())
        }
    }

    fn sample_disney_clearcoat(&self) -> Option<UnitVec3> {
        let v_out = self.v_out;

        let a: f64 = 0.25;
        let a2 = a * a;

        let r0 = Random::f64();
        let r1 = Random::f64();
        let cos_theta = ((1.0 - a2.powf(1.0 - r0)) / (1.0 - a2)).max(0.0).sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let phi = 2.0 * PI * r1;

        let mut v_half = UnitVec3::from_vec3_raw(Vec3::new(
            sin_theta * phi.cos(),
            cos_theta,
            sin_theta * phi.sin(),
        ));
        if v_half.dot(&v_out) < 0.0 {
            v_half = -v_half;
        }

        let v_in = v_out.reflect2(&v_half);
        if v_in.dot(&v_out) < 0.0 {
            None
        } else {
            Some(UnitVec3::from_vec3(self.uvw.onb_to_world(v_in)).unwrap())
        }
    }

    fn sample_disney_diffuse(&self) -> Option<UnitVec3> {
        let v_out = &self.v_out;
        let sign = v_out.cos_theta().signum();

        let mut v_in = UnitVec3::from_vec3_raw(sign * UnitVec3::random_cosine_direction().into_inner());
        if Random::f64() <= self.material.diff_trans {
            v_in = -v_in;
        }

        let dot_nl = v_in.cos_theta();
        if dot_nl == 0.0 {
            None
        } else {
            Some(UnitVec3::from_vec3(self.uvw.onb_to_world(v_in.into_inner())).unwrap())
        }
    }

    fn disney_spec_transmission(&self) -> Option<UnitVec3> {
        let v_out = &self.v_out;

        if v_out.cos_theta() == 0.0 {
            return None;
        }

        let rscaled = if self.thin {
            thin_transmission_roughness(self.material.ior, self.material.roughness)
        } else {
            self.material.roughness
        };

        let (tax, tay) = calculate_anisotropic_params(rscaled, self.material.anisotropic);

        let r0 = Random::f64();
        let r1 = Random::f64();
        let v_half = sample_ggx_vndf_anisotropic(v_out, tax, tay, r0, r1);

        let mut dot_vh = v_out.dot(&v_half);
        if v_half.y() < 0.0 {
            dot_vh = -dot_vh;
        }

        let ni = if v_out.y() > 0.0 {
            1.0
        } else {
            self.material.ior
        };
        let nt = if v_out.y() > 0.0 {
            self.material.ior
        } else {
            1.0
        };
        let relative_ior = ni / nt;

        let f = dielectric(dot_vh, 1.0, self.material.ior);

        let v_in = if Random::f64() <= f {
            UnitVec3::from_vec3(v_out.reflect2(&v_half)).unwrap()
        } else {
            if self.thin {
                let wi = v_out.reflect2(&v_half);
                UnitVec3::new(wi.x(), -wi.y(), wi.z()).unwrap()
            } else {
                if let Some(wi) = v_out.refract2(&v_half, relative_ior) {
                    wi
                } else {
                    UnitVec3::from_vec3(v_out.reflect2(&v_half)).unwrap()
                }
            }
        };

        if v_in.cos_theta() == 0.0 {
            None
        } else {
            Some(UnitVec3::from_vec3(self.uvw.onb_to_world(v_in.into_inner())).unwrap())
        }
    }
}

impl<'a> PDF for DisneyPDF<'a> {
    fn value(&self, direction: &Vec3) -> (Color, f64) {
        let v_in = UnitVec3::from_vec3_raw(
            self.uvw
                .world_to_onb(UnitVec3::from_vec3(*direction).unwrap().into_inner()),
        );
        let (attentuation, f_pdf, _) = self.material.evaluate_disney(&self.v_out, &v_in, self.thin);
        (attentuation, f_pdf)
    }

    fn generate(&self) -> Option<UnitVec3> {
        let (p_specular, p_diffuse, p_clearcoat, p_transmission) =
            self.material.calculate_lobe_pdfs();

        let p = Random::f64();

        if p <= p_specular {
            self.sample_disney_brdf()
        } else if p <= p_specular + p_clearcoat {
            self.sample_disney_clearcoat()
        } else if p <= p_specular + p_diffuse + p_clearcoat {
            self.sample_disney_diffuse()
        } else if p_transmission >= 0.0 {
            self.disney_spec_transmission()
        } else {
            panic!("The conditions should be exhausted!");
        }
    }
}

fn sample_ggx_vndf_anisotropic(v_out: &UnitVec3, ax: f64, ay: f64, u1: f64, u2: f64) -> UnitVec3 {
    let v = UnitVec3::new(v_out.x() * ax, v_out.y(), v_out.z() * ay).unwrap();

    let t1 = if v.y() < 0.9999999 { // 此处的突变会导致分层，把右侧设的非常接近 1 就看不到分层了
        UnitVec3::from_vec3_raw(v.cross(&UnitVec3::Y_AXIS))
    } else {
        UnitVec3::X_AXIS
    };
    let t2 = t1.cross(&v);

    let a = 1.0 / (1.0 + v.y());
    let r = u1.sqrt();
    let phi = if u2 < a {
        (u2 / a) * PI
    } else {
        PI + (u2 - a) / (1.0 - a) * PI
    };
    let p1 = r * phi.cos();
    let p2 = r * phi.sin() * (if u2 < a { 1.0 } else { v.y() });

    let n = p1 * t1.as_inner() + p2 * t2 + (1.0 - p1 * p1 - p2 * p2).max(0.0).sqrt() * v.as_inner();

    UnitVec3::from_vec3(Vec3::new(ax * n.x(), n.y(), ay * n.z())).unwrap()
}
