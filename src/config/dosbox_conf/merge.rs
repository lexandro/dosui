//! Inheritance: `effective = defaults.merge(profile)`.

use super::DosboxConfig;

impl DosboxConfig {
    /// Layer `overrides` on top of `self` (the defaults): every set leaf in
    /// `overrides` wins; unset (`None`) leaves inherit from `self`. Passthrough
    /// maps merge per (section, key), with `overrides` winning.
    pub fn merge(&self, overrides: &DosboxConfig) -> DosboxConfig {
        let pick = |o: &Option<String>, d: &Option<String>| o.clone().or_else(|| d.clone());

        let mut passthrough = self.passthrough.clone();
        for (section, keys) in &overrides.passthrough {
            let target = passthrough.entry(section.clone()).or_default();
            for (key, value) in keys {
                target.insert(key.clone(), value.clone());
            }
        }

        DosboxConfig {
            output: pick(&overrides.output, &self.output),
            fullscreen: overrides.fullscreen.or(self.fullscreen),
            vsync: pick(&overrides.vsync, &self.vsync),
            machine: pick(&overrides.machine, &self.machine),
            memsize: overrides.memsize.or(self.memsize),
            vmemsize: pick(&overrides.vmemsize, &self.vmemsize),
            xms: overrides.xms.or(self.xms),
            ems: pick(&overrides.ems, &self.ems),
            umb: overrides.umb.or(self.umb),
            core: pick(&overrides.core, &self.core),
            cputype: pick(&overrides.cputype, &self.cputype),
            cycles: pick(&overrides.cycles, &self.cycles),
            aspect: overrides.aspect.or(self.aspect),
            glshader: pick(&overrides.glshader, &self.glshader),
            sbtype: pick(&overrides.sbtype, &self.sbtype),
            oplmode: pick(&overrides.oplmode, &self.oplmode),
            sbbase: pick(&overrides.sbbase, &self.sbbase),
            sbirq: pick(&overrides.sbirq, &self.sbirq),
            sbdma: pick(&overrides.sbdma, &self.sbdma),
            sbhdma: pick(&overrides.sbhdma, &self.sbhdma),
            rate: overrides.rate.or(self.rate),
            gus: overrides.gus.or(self.gus),
            gusbase: pick(&overrides.gusbase, &self.gusbase),
            gusirq: pick(&overrides.gusirq, &self.gusirq),
            gusdma: pick(&overrides.gusdma, &self.gusdma),
            mididevice: pick(&overrides.mididevice, &self.mididevice),
            mpu401: pick(&overrides.mpu401, &self.mpu401),
            soundfont: pick(&overrides.soundfont, &self.soundfont),
            pcspeaker: pick(&overrides.pcspeaker, &self.pcspeaker),
            tandy: pick(&overrides.tandy, &self.tandy),
            keyboardlayout: pick(&overrides.keyboardlayout, &self.keyboardlayout),
            mouse_capture: pick(&overrides.mouse_capture, &self.mouse_capture),
            mouse_sensitivity: pick(&overrides.mouse_sensitivity, &self.mouse_sensitivity),
            joysticktype: pick(&overrides.joysticktype, &self.joysticktype),
            joy_autofire: overrides.joy_autofire.or(self.joy_autofire),
            joy_swap34: overrides.joy_swap34.or(self.joy_swap34),
            dos_ver: pick(&overrides.dos_ver, &self.dos_ver),
            country: pick(&overrides.country, &self.country),
            passthrough,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn merge_overrides_win_and_unset_inherit() {
        let defaults = DosboxConfig {
            cycles: Some("auto".into()),
            memsize: Some(16),
            output: Some("opengl".into()),
            ..Default::default()
        };
        let overrides = DosboxConfig {
            cycles: Some("max".into()), // wins
            memsize: None,              // inherits 16
            ..Default::default()
        };
        let effective = defaults.merge(&overrides);
        assert_eq!(effective.cycles.as_deref(), Some("max"));
        assert_eq!(effective.memsize, Some(16));
        assert_eq!(effective.output.as_deref(), Some("opengl"));
    }

    #[test]
    fn merge_passthrough_combines_with_override_winning() {
        let mut defaults = DosboxConfig::default();
        let mut d = IndexMap::new();
        d.insert("glshader".to_string(), "crt".to_string());
        d.insert("aspect".to_string(), "true".to_string());
        defaults.passthrough.insert("render".to_string(), d);

        let mut overrides = DosboxConfig::default();
        let mut o = IndexMap::new();
        o.insert("glshader".to_string(), "sharp".to_string()); // wins
        overrides.passthrough.insert("render".to_string(), o);

        let effective = defaults.merge(&overrides);
        assert_eq!(effective.passthrough["render"]["glshader"], "sharp");
        assert_eq!(effective.passthrough["render"]["aspect"], "true");
    }
}
