//! STAGE offset readback through libmonado.
//! Playspace movers (WayVR drag, floor fix, recenter) move STAGE with
//! mnd_root_set_reference_space_offset. WiVRn publishes SolarXR trackers in the HMD
//! tracking origin, so Rebocap must see the head there too, not in the moved STAGE.
use openxr as xr;
use std::{
    env,
    ffi::{c_char, c_void, CStr, CString},
    fs,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    ptr,
};

// monado.h: mnd_pose_t is orientation x, y, z, w followed by position x, y, z.
#[repr(C)]
#[derive(Default)]
struct MndPose {
    orientation: [f32; 4],
    position: [f32; 3],
}

const MND_SPACE_REFERENCE_TYPE_STAGE: i32 = 3;

type Create = unsafe extern "C" fn(*mut *mut c_void) -> i32;
type Destroy = unsafe extern "C" fn(*mut *mut c_void);
type GetOffset = unsafe extern "C" fn(*mut c_void, i32, *mut MndPose) -> i32;

pub struct Stage {
    lib: *mut c_void,
    root: *mut c_void,
    destroy: Destroy,
    get_offset: GetOffset,
}

fn libmonado_path() -> Result<PathBuf, String> {
    let json = PathBuf::from(env::var_os("XR_RUNTIME_JSON").ok_or("XR_RUNTIME_JSON is not set")?);
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&json).map_err(|e| format!("{}: {e}", json.display()))?,
    )
    .map_err(|e| format!("{}: {e}", json.display()))?;
    let path = manifest["runtime"]["MND_libmonado_path"]
        .as_str()
        .ok_or_else(|| format!("{} has no MND_libmonado_path", json.display()))?;
    Ok(json.parent().unwrap_or(json.as_path()).join(path))
}

fn dlerror() -> String {
    let e = unsafe { libc::dlerror() };
    if e.is_null() {
        "unknown dlopen error".into()
    } else {
        unsafe { CStr::from_ptr(e) }.to_string_lossy().into_owned()
    }
}

impl Stage {
    pub fn connect() -> Result<Self, String> {
        let path = libmonado_path()?;
        let name = CString::new(path.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
        let lib = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
        if lib.is_null() {
            return Err(dlerror());
        }
        let symbol = |name: &CStr| {
            let f = unsafe { libc::dlsym(lib, name.as_ptr() as *const c_char) };
            (!f.is_null()).then_some(f).ok_or_else(dlerror)
        };
        let result = (|| -> Result<Self, String> {
            let create: Create = unsafe { std::mem::transmute(symbol(c"mnd_root_create")?) };
            let destroy: Destroy = unsafe { std::mem::transmute(symbol(c"mnd_root_destroy")?) };
            let get_offset: GetOffset =
                unsafe { std::mem::transmute(symbol(c"mnd_root_get_reference_space_offset")?) };
            let mut root = ptr::null_mut();
            let code = unsafe { create(&mut root) };
            if code != 0 || root.is_null() {
                return Err(format!("mnd_root_create failed ({code})"));
            }
            Ok(Self {
                lib,
                root,
                destroy,
                get_offset,
            })
        })();
        if result.is_err() {
            unsafe { libc::dlclose(lib) };
        }
        result
    }

    /// Pose of STAGE in the tracking origin.
    pub fn offset(&self) -> Result<xr::Posef, i32> {
        let mut p = MndPose::default();
        let code =
            unsafe { (self.get_offset)(self.root, MND_SPACE_REFERENCE_TYPE_STAGE, &mut p) };
        if code != 0 {
            return Err(code);
        }
        let [x, y, z, w] = p.orientation;
        let [px, py, pz] = p.position;
        Ok(xr::Posef {
            orientation: xr::Quaternionf { x, y, z, w },
            position: xr::Vector3f {
                x: px,
                y: py,
                z: pz,
            },
        })
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        unsafe {
            (self.destroy)(&mut self.root);
            libc::dlclose(self.lib);
        }
    }
}

/// a * b: b expressed in a's parent space.
pub fn compose(a: xr::Posef, b: xr::Posef) -> xr::Posef {
    let q = a.orientation;
    let v = b.position;
    // v + w*t + q.xyz × t, with t = 2 * (q.xyz × v)
    let t = [
        2.0 * (q.y * v.z - q.z * v.y),
        2.0 * (q.z * v.x - q.x * v.z),
        2.0 * (q.x * v.y - q.y * v.x),
    ];
    let r = b.orientation;
    xr::Posef {
        position: xr::Vector3f {
            x: a.position.x + v.x + q.w * t[0] + (q.y * t[2] - q.z * t[1]),
            y: a.position.y + v.y + q.w * t[1] + (q.z * t[0] - q.x * t[2]),
            z: a.position.z + v.z + q.w * t[2] + (q.x * t[1] - q.y * t[0]),
        },
        orientation: xr::Quaternionf {
            w: q.w * r.w - q.x * r.x - q.y * r.y - q.z * r.z,
            x: q.w * r.x + q.x * r.w + q.y * r.z - q.z * r.y,
            y: q.w * r.y - q.x * r.z + q.y * r.w + q.z * r.x,
            z: q.w * r.z + q.x * r.y - q.y * r.x + q.z * r.w,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_applies_rotation_then_translation() {
        let h = std::f32::consts::FRAC_1_SQRT_2;
        // STAGE moved 1 m along +X and turned 90° about +Y.
        let stage = xr::Posef {
            orientation: xr::Quaternionf { x: 0.0, y: h, z: 0.0, w: h },
            position: xr::Vector3f { x: 1.0, y: 0.0, z: 0.0 },
        };
        let head = xr::Posef {
            orientation: xr::Quaternionf::IDENTITY,
            position: xr::Vector3f { x: 0.0, y: 1.6, z: -1.0 },
        };
        let p = compose(stage, head);
        // -Z rotated +90° about +Y is -X.
        assert!((p.position.x - 0.0).abs() < 1e-6);
        assert!((p.position.y - 1.6).abs() < 1e-6);
        assert!((p.position.z - 0.0).abs() < 1e-6);
        assert!((p.orientation.y - h).abs() < 1e-6 && (p.orientation.w - h).abs() < 1e-6);
        let p = compose(xr::Posef::IDENTITY, head);
        assert_eq!(p.position.z, -1.0);
    }
}
