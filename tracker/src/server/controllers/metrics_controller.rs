use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::metrics::GroupBy;
use crate::metrics::MetricsSender;
use crate::metrics::TimeFrame;
use crate::server::errors::MetricsError;

pub struct MetricsController;

impl MetricsController {
    pub fn handler_metrics(
        mut http_service: Box<dyn IHttpService>,
        _request: HttpGetRequest,
        metrics: MetricsSender,
    ) -> Result<(), MetricsError> {
        let json: String = metrics.get_metrics_response(
            "torrents".to_string(),
            TimeFrame::LastHours(1),
            GroupBy::Minutes(1),
        )?;

        http_service.send_ok_response(json.as_bytes().to_vec(), "application/json".to_string())?;

        Ok(())
    }
}
