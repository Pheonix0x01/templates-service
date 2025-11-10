use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::{ok, Ready};
use futures::Future;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, HistogramVec,
};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

lazy_static::lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "templates_requests_total",
        "Total number of HTTP requests",
        &["method", "route", "status"]
    )
    .unwrap();

    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "templates_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "route"]
    )
    .unwrap();

    pub static ref TEMPLATE_CACHE_HITS: CounterVec = register_counter_vec!(
        "templates_cache_hits_total",
        "Total number of cache hits",
        &["cache_type"]
    )
    .unwrap();

    pub static ref TEMPLATE_CACHE_MISSES: CounterVec = register_counter_vec!(
        "templates_cache_misses_total",
        "Total number of cache misses",
        &["cache_type"]
    )
    .unwrap();

    pub static ref TEMPLATE_RENDER_DURATION: HistogramVec = register_histogram_vec!(
        "templates_render_duration_seconds",
        "Template rendering duration in seconds",
        &["template_type"]
    )
    .unwrap();

    pub static ref DB_QUERIES_TOTAL: CounterVec = register_counter_vec!(
        "templates_db_queries_total",
        "Total number of database queries",
        &["operation"]
    )
    .unwrap();
}

pub struct Metrics;

impl<S, B> Transform<S, ServiceRequest> for Metrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddleware { service })
    }
}

pub struct MetricsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed().as_secs_f64();
            let status = res.status().as_u16().to_string();

            HTTP_REQUESTS_TOTAL
                .with_label_values(&[&method, &path, &status])
                .inc();

            HTTP_REQUEST_DURATION
                .with_label_values(&[&method, &path])
                .observe(duration);

            Ok(res)
        })
    }
}