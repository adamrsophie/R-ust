use r_ust_sexp::SexpType;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SexpHandle(usize);

impl SexpHandle {
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcConfig {
    pub initial_ncells: usize,
    pub initial_vcells: usize,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            initial_ncells: 350_000,
            initial_vcells: 8_388_608,
        }
    }
}

#[derive(Debug, Default)]
pub struct ProtectStack {
    handles: Vec<SexpHandle>,
}

impl ProtectStack {
    pub fn protect(&mut self, handle: SexpHandle) -> SexpHandle {
        self.handles.push(handle);
        handle
    }

    pub fn unprotect(&mut self, count: usize) {
        let keep = self
            .handles
            .len()
            .checked_sub(count)
            .expect("R unprotect count exceeds protect stack depth");
        self.handles.truncate(keep);
    }

    pub fn depth(&self) -> usize {
        self.handles.len()
    }

    pub fn as_slice(&self) -> &[SexpHandle] {
        &self.handles
    }
}

#[derive(Debug)]
pub struct Gc {
    config: GcConfig,
    protected: ProtectStack,
    allocated_types: Vec<SexpType>,
}

impl Gc {
    pub fn new(config: GcConfig) -> Self {
        Self {
            config,
            protected: ProtectStack::default(),
            allocated_types: Vec::new(),
        }
    }

    pub fn alloc_header(&mut self, sexptype: SexpType) -> SexpHandle {
        let handle = SexpHandle::from_index(self.allocated_types.len());
        self.allocated_types.push(sexptype);
        handle
    }

    pub fn protect(&mut self, handle: SexpHandle) -> SexpHandle {
        self.protected.protect(handle)
    }

    pub fn unprotect(&mut self, count: usize) {
        self.protected.unprotect(count);
    }

    pub fn protect_depth(&self) -> usize {
        self.protected.depth()
    }

    pub const fn config(&self) -> GcConfig {
        self.config
    }
}

impl Default for Gc {
    fn default() -> Self {
        Self::new(GcConfig::default())
    }
}
