#[derive(Clone, Debug, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "En attente", Self::Running => "En cours",
            Self::Completed => "Terminé", Self::Failed => "Échoué", Self::Cancelled => "Annulé",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum JobPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl JobPriority {
    pub fn label(&self) -> &'static str {
        match self { Self::Low => "Faible", Self::Normal => "Normal", Self::High => "Haute", Self::Critical => "Critique" }
    }
    pub fn value(&self) -> u8 {
        match self { Self::Low => 0, Self::Normal => 1, Self::High => 2, Self::Critical => 3 }
    }
    pub const ALL: [JobPriority; 4] = [JobPriority::Low, JobPriority::Normal, JobPriority::High, JobPriority::Critical];
}

#[derive(Clone, Debug)]
pub struct JobHandle {
    pub id: u64,
    pub name: String,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub progress: f64,
    pub elapsed_ms: f64,
    pub dependency: Option<u64>,
}

impl JobHandle {
    pub fn new(id: u64, name: impl Into<String>, priority: JobPriority) -> Self {
        Self { id, name: name.into(), priority, status: JobStatus::Pending, progress: 0.0, elapsed_ms: 0.0, dependency: None }
    }
}

#[derive(Clone, Debug)]
pub struct JobScheduler {
    pub jobs: Vec<JobHandle>,
    pub worker_count: usize,
    pub max_jobs_per_frame: usize,
    pub enabled: bool,
    pub total_completed: u64,
    pub total_failed: u64,
    next_id: u64,
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            worker_count: 4,
            max_jobs_per_frame: 8,
            enabled: true,
            total_completed: 0,
            total_failed: 0,
            next_id: 0,
        }
    }
}

impl JobScheduler {
    pub fn new() -> Self { Self::default() }

    pub fn schedule(&mut self, name: impl Into<String>, priority: JobPriority) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut job = JobHandle::new(id, name, priority);
        job.status = JobStatus::Pending;
        self.jobs.push(job);
        self.sort_by_priority();
        id
    }

    pub fn schedule_after(&mut self, name: impl Into<String>, priority: JobPriority, after: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut job = JobHandle::new(id, name, priority);
        job.dependency = Some(after);
        self.jobs.push(job);
        self.sort_by_priority();
        id
    }

    pub fn cancel(&mut self, id: u64) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            if job.status == JobStatus::Pending { job.status = JobStatus::Cancelled; }
        }
    }

    pub fn tick(&mut self, dt: f64) {
        if !self.enabled { return; }
        let completed_ids: Vec<u64> = self.jobs.iter()
            .filter(|j| j.status == JobStatus::Completed)
            .map(|j| j.id)
            .collect();
        let mut dispatched = 0;
        for job in &mut self.jobs {
            if dispatched >= self.max_jobs_per_frame { break; }
            let dep_ok = job.dependency.map(|dep_id| completed_ids.contains(&dep_id)).unwrap_or(true);
            if job.status == JobStatus::Pending && dep_ok {
                job.status = JobStatus::Running;
                dispatched += 1;
            }
            if job.status == JobStatus::Running {
                job.progress = (job.progress + dt * 2.0).min(1.0);
                job.elapsed_ms += dt * 1000.0;
                if job.progress >= 1.0 {
                    job.status = JobStatus::Completed;
                    self.total_completed += 1;
                }
            }
        }
        self.jobs.retain(|j| !matches!(j.status, JobStatus::Completed | JobStatus::Cancelled | JobStatus::Failed) || j.elapsed_ms < 2000.0);
    }

    pub fn sort_by_priority(&mut self) {
        self.jobs.sort_by(|a, b| b.priority.value().cmp(&a.priority.value()));
    }

    pub fn pending_count(&self) -> usize {
        self.jobs.iter().filter(|j| j.status == JobStatus::Pending).count()
    }

    pub fn running_count(&self) -> usize {
        self.jobs.iter().filter(|j| j.status == JobStatus::Running).count()
    }
}
