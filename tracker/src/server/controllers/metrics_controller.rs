use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::metrics::GroupBy;
use crate::metrics::MetricsSender;
use crate::metrics::TimeFrame;
use crate::server::errors::MetricsError;

const METRIC_KEY: &str = "key";
const TIME_FRAME_INTERVAL_KEY: &str = "timeFrameInterval";
const TIME_FRAME_COUNT_KEY: &str = "timeFrameCount";
const GROUP_BY_KEY: &str = "groupBy";
const GROUP_BY_COUNT_KEY: &str = "groupByCount";

pub struct MetricsController;

impl MetricsController {
    pub fn handler_metrics(
        mut http_service: Box<dyn IHttpService>,
        request: HttpGetRequest,
        metrics: MetricsSender,
    ) -> Result<(), MetricsError> {
        // get key param from request, otherwise throw error MetricsError::MissingKey
        let metric_key = request
            .params
            .get(METRIC_KEY)
            .ok_or(MetricsError::MissingKey(METRIC_KEY.to_owned()))?;
        let time_frame_key = request
            .params
            .get(TIME_FRAME_INTERVAL_KEY)
            .ok_or(MetricsError::MissingKey(TIME_FRAME_INTERVAL_KEY.to_owned()))?;
        let time_frame_count_key = request
            .params
            .get(TIME_FRAME_COUNT_KEY)
            .ok_or(MetricsError::MissingKey(TIME_FRAME_COUNT_KEY.to_owned()))?;
        let groupby_key = request
            .params
            .get(GROUP_BY_KEY)
            .ok_or(MetricsError::MissingKey(GROUP_BY_KEY.to_owned()))?;
        let groupby_count_key = request
            .params
            .get(GROUP_BY_COUNT_KEY)
            .ok_or(MetricsError::MissingKey(GROUP_BY_COUNT_KEY.to_owned()))?;

        let json: String = metrics.get_metrics_response(
            metric_key.to_owned(),
            TimeFrame::LastHours(1),
            GroupBy::Minutes(1),
        )?;

        http_service.send_ok_response(json.as_bytes().to_vec(), "application/json".to_string())?;

        Ok(())
    }
}
