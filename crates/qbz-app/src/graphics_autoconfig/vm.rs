pub(super) fn is_virtual_machine() -> bool {
    if let Ok(product) = std::fs::read_to_string("/sys/class/dmi/id/product_name") {
        let p = product.trim().to_lowercase();
        if p.contains("virtualbox")
            || p.contains("vmware")
            || p.contains("qemu")
            || p.contains("bochs")
            || p.contains("hyper-v")
        {
            return true;
        }
    }
    if let Ok(vendor) = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor") {
        let v = vendor.trim().to_lowercase();
        if v.contains("innotek")
            || v.contains("vmware")
            || v.contains("qemu")
            || v.contains("xen")
            || v.contains("parallels")
        {
            return true;
        }
    }
    if let Ok(h) = std::fs::read_to_string("/sys/hypervisor/type") {
        if !h.trim().is_empty() {
            return true;
        }
    }
    false
}
