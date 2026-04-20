use crate::core::engine::rendering::raytracing::Material;

use super::builder::MaterialBuilder;
use super::spectrum::Spectrum;

pub fn diffuse(r: f64, g: f64, b: f64, roughness: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .roughness(roughness)
        .build()
}

pub fn metal(r: f64, g: f64, b: f64, roughness: f64, reflectivity: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .roughness(roughness)
        .metallic(1.0)
        .reflectivity(reflectivity)
        .build()
}

pub fn dielectric(r: f64, g: f64, b: f64, ior: f64, transmission: f64, roughness: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .roughness(roughness)
        .transmission(transmission)
        .ior(ior)
        .build()
}

pub fn emissive(r: f64, g: f64, b: f64, strength: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .emission_rgb(r * strength, g * strength, b * strength)
        .roughness(0.05)
        .build()
}

pub fn emissive_temperature(kelvin: f64, peak_power: f64) -> Material {
    MaterialBuilder::new()
        .albedo_temperature(kelvin, 1.0)
        .emission_temperature(kelvin, peak_power)
        .roughness(0.04)
        .build()
}

pub fn subsurface(r: f64, g: f64, b: f64, roughness: f64, sss: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .roughness(roughness)
        .subsurface(sss)
        .build()
}

pub fn clearcoat(r: f64, g: f64, b: f64, coat: f64, roughness: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .roughness(roughness)
        .clearcoat(coat)
        .build()
}

pub fn iridescent(r: f64, g: f64, b: f64, iridescence: f64, roughness: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .roughness(roughness)
        .iridescence(iridescence)
        .build()
}

pub fn anisotropic(r: f64, g: f64, b: f64, roughness: f64, aniso: f64) -> Material {
    MaterialBuilder::new()
        .albedo_rgb(r, g, b)
        .roughness(roughness)
        .anisotropy(aniso)
        .build()
}

pub fn spectral_wavelength(wavelength_nm: f64, power: f64, spread_nm: f64) -> Material {
    MaterialBuilder::new()
        .albedo_spectrum(Spectrum::from_wavelength(wavelength_nm, power, spread_nm))
        .build()
}

pub fn spectral_black_body(kelvin: f64, albedo_peak: f64, emission_peak: f64) -> Material {
    MaterialBuilder::new()
        .albedo_temperature(kelvin, albedo_peak)
        .emission_temperature(kelvin, emission_peak)
        .build()
}
