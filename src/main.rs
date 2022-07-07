mod units;

use serde::{Deserialize, Serialize};
use std::ops::Range;
use units::{Meters, Yards};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FSXChallenge {
    name: String,
    num_stations: usize,
    #[serde(rename = "Station")]
    stations: Vec<Station>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Station {
    array_index: usize,
    desc: String,
    station_num: usize,
    skill_type: usize,
    num_shots_am: usize,
    num_shots_pro: usize,
    num_shots_to_use: usize,
    trgt_dist_women: Meters,
    trgt_dist_am: Meters,
    trgt_dist_pro: Meters,
    inner_ring_diam_am: Meters,
    mid_ring_diam_am: Meters,
    outer_ring_diam_am: Meters,
    inner_ring_diam_pro: Meters,
    mid_ring_diam_pro: Meters,
    outer_ring_diam_pro: Meters,
    inner_score: usize,
    mid_score: usize,
    outer_score: usize,
    obstacle: usize,
    obstacle_dist: Meters,
}

fn yards_within(bounds: Range<Yards>, min_gap: Yards) -> RandYardsIter {
    RandYardsIter {
        rng: ::rand::thread_rng(),
        bounds,
        min_gap,
        last: None,
    }
}
struct RandYardsIter {
    rng: ::rand::prelude::ThreadRng,
    bounds: Range<Yards>,
    min_gap: Yards,
    last: Option<Yards>,
}
impl Iterator for RandYardsIter {
    type Item = Yards;
    fn next(&mut self) -> Option<Self::Item> {
        use ::rand::prelude::Rng;
        loop {
            let y = self.rng.gen_range(self.bounds.clone());
            match self.last {
                Some(ly) if ly.abs_diff(y) < self.min_gap => continue,
                _ => {
                    self.last = Some(y);
                    return Some(y);
                }
            }
        }
    }
}

const NUM_STATIONS: usize = 20;

fn new_random_challenge(
    dist: Range<Yards>,
    min_gap: Yards,
    inner_ring: Yards,
    mid_ring: Yards,
    outer_ring: Yards,
    inner_score: usize,
    mid_score: usize,
    outer_score: usize,
) -> FSXChallenge {
    let stations: Vec<_> = yards_within(dist.clone(), min_gap)
        .take(NUM_STATIONS)
        .enumerate()
        .map(|(idx, yds)| Station {
            array_index: idx,
            desc: "1".to_owned(),
            station_num: idx + 1,
            skill_type: 0,
            num_shots_am: 1,
            num_shots_pro: 1,
            num_shots_to_use: 1,
            trgt_dist_women: yds.to_meters(),
            trgt_dist_am: yds.to_meters(),
            trgt_dist_pro: yds.to_meters(),
            inner_ring_diam_am: inner_ring.to_meters(),
            mid_ring_diam_am: mid_ring.to_meters(),
            outer_ring_diam_am: outer_ring.to_meters(),
            inner_ring_diam_pro: inner_ring.to_meters(),
            mid_ring_diam_pro: mid_ring.to_meters(),
            outer_ring_diam_pro: outer_ring.to_meters(),
            inner_score,
            mid_score,
            outer_score,
            obstacle: 0,
            obstacle_dist: Yards::new(0).to_meters(),
        })
        .collect();
    let uid = {
        use std::hash::{Hash, Hasher};
        let mut s = std::collections::hash_map::DefaultHasher::new();
        for station in &stations {
            station.trgt_dist_am.to_yards().hash(&mut s);
        }
        s.finish() as u32
    };
    let min = dist.start.as_float();
    let max = dist.end.as_float();
    let name = format!(r"{min:.0} - {max:.0} {uid:08x}");

    FSXChallenge {
        name,
        num_stations: NUM_STATIONS,
        stations,
    }
}

use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(rand_challenge));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct ChallengeInput {
    min: Yards,
    max: Yards,
    min_gap: Option<Yards>,
    inner_ring: Option<Yards>,
    mid_ring: Option<Yards>,
    outer_ring: Option<Yards>,
    inner_score: Option<usize>,
    mid_score: Option<usize>,
    outer_score: Option<usize>,
}
// basic handler that responds with a static string
async fn rand_challenge(input: Option<Query<ChallengeInput>>) -> impl IntoResponse {
    match input {
        Some(Query(ChallengeInput {
            min,
            max,
            min_gap,
            inner_ring,
            mid_ring,
            outer_ring,
            inner_score,
            mid_score,
            outer_score,
        })) => {
            let challenge = new_random_challenge(
                min..max,
                min_gap.unwrap_or_else(|| Yards::new(10)),
                inner_ring.unwrap_or_else(|| Yards::new(8)),
                mid_ring.unwrap_or_else(|| Yards::new(16)),
                outer_ring.unwrap_or_else(|| Yards::new(24)),
                inner_score.unwrap_or(5),
                mid_score.unwrap_or(3),
                outer_score.unwrap_or(1),
            );
            let filename = format!("{}.xml", challenge.name);
            match quick_xml::se::to_string(&challenge) {
                Ok(challenge) => (
                    StatusCode::OK,
                    [("Content-Type", "application/xml")],
                    [(
                        "Content-Disposition",
                        format!(r#"inline; filename="{filename}""#),
                    )],
                    challenge,
                )
                    .into_response(),
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
                }
            }
        }
        None => INPUT_FORM.into_response(),
    }
}

const INPUT_FORM: Html<&str> = Html(
    r#"
      <!doctype html>
      <html>
          <head></head>
          <body>
              <form >
                  <label for="min">
                      Min Yardage:
                      <input type="text" name="min">
                  </label>
                  <label for="max">
                      Max Yardage:
                      <input type="text" name="max">
                  </label>
                  <label for="min_gap">
                      Max Yardage:
                      <input value=10 type="text" name="min_gap">
                  </label>
                  <label for="inner_ring">
                      Inner Ring Yardage:
                      <input value=8 type="text" name="inner_ring">
                  </label>
                  <label for="mid_ring">
                      Mid Ring Yardage:
                      <input value=16 type="text" name="mid_ring">
                  </label>
                  <label for="outer_ring">
                      Outer Ring Yardage:
                      <input value=24 type="text" name="outer_ring">
                  </label>
                  <label for="inner_score">
                      Inner Score:
                      <input value=5 type="text" name="inner_score">
                  </label>
                  <label for="mid_score">
                      Mid Score:
                      <input value=3 type="text" name="mid_score">
                  </label>
                  <label for="outer_score">
                      Outer Score:
                      <input value=1 type="text" name="outer_score">
                  </label>
                  <input type="submit">
              </form>
          </body>
      </html>
    "#,
);
