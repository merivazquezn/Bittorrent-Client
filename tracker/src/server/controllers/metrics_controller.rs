use crate::http::HttpError;
use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::metrics::GroupBy;
use crate::metrics::MetricsSender;
use crate::metrics::TimeFrame;

pub struct MetricsController;

impl MetricsController {
    pub fn handler_metrics(
        http_service: Box<dyn IHttpService>,
        _request: HttpGetRequest,
        metrics: MetricsSender,
    ) -> Result<(), HttpError> {
        metrics.send_metric(
            http_service,
            "torrents".to_string(),
            TimeFrame::LastHours(1),
            GroupBy::Minutes(1),
        );
        // http_service.send_ok_response(
        //     "{ status: \"ok\" }".as_bytes().to_vec(),
        //     "application/json".to_string(),
        // )?;

        Ok(())
    }
}
