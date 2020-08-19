use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Serialize, Deserialize)]
struct Points {
    points: Vec<(usize, usize)>,
    token: String,
}

#[derive(Serialize, Deserialize)]
struct ComputedResult {
    points: Vec<usize>,
    token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let endpoint: &str = "http://13.74.31.101/api/points";

    println!("Retrieving data from endpoint");
    let body = reqwest::get(endpoint).await?
        .text()
        .await?;

    let parsed_points: Points = serde_json::from_str(&body)?;
    println!("Token and frame balls: {:?}, {:?}", parsed_points.token, parsed_points.points);

    let computed_score: Vec<usize> = compute_score(parsed_points.points);

    let computed_result: ComputedResult = ComputedResult {token: parsed_points.token, points: computed_score};
    println!("Response sent to endpoint: {:?}", serde_json::to_string(&computed_result)?);
    let client: reqwest::Client = reqwest::Client::new();
    let post_response = client.post(endpoint).json(&computed_result).send().await?;

    if post_response.status() == 200 {
        println!("Successfully calculated bowling results")
    } else {
        println!("Failed to calculate result :(")
    }

    Ok(())
}

fn compute_score(points: Vec<(usize, usize)>) -> Vec<usize> {
    let mut result: Vec<usize> = vec![];
    let mut previous_result: usize = 0;

    for i in 0..points.len() {
        let frame_score = points[i].0 + points[i].1;

        //last frame
        if i == points.len() - 1 {
            result.push(previous_result + frame_score);
            break;
        }

        //normal frame
        if points[i].0 + points[i].1 != 10 {
            result.push(previous_result + frame_score);
            previous_result += frame_score;
            continue;
        }

        //remember strikes get bonus pin fall on the next two rolls
        if points[i].0 == 10 {
            if points.len() - 1 == 1 {
                result.push(previous_result + 10 + points[i + 1].0 + points[i + 1].1);
                previous_result += 10 + points[i + 1].0 + points[i + 1].1;
            } else {
                //test if endgame
                if points[i + 1].0 == 10 && points[i + 1].1 == 10 {
                    result.push(previous_result + 10 + points[i + 1].0 + points[i + 1].1);
                    break;
                } else if points[i + 1].0 == 10 {
                    result.push(previous_result + 10 + points[i + 1].0 + points[i + 2].0);
                    previous_result += 10 + points[i + 1].0 + points[i + 2].0;
                } else if points[i + 1].0 != 10 {
                    result.push(previous_result + 10 + points[i + 1].0 + points[i + 1].1);
                    previous_result += 10 + points[i + 1].0 + points[i + 1].1;
                }
            }
            continue;
        }

        //spare frame
        if frame_score == 10 {
            result.push(previous_result + frame_score + points[i + 1].0);
            previous_result += frame_score + points[i + 1].0;
            continue;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_score() {
        assert_eq!(vec![3, 10, 10],
                   compute_score(vec![(1, 2), (3, 4), (0, 0)]));
    }

    #[test]
    fn test_spare() {
        assert_eq!(vec![15, 28, 32],
                   compute_score(vec![(3, 7), (5, 5), (3, 1)]));
    }

    #[test]
    fn test_another_spare() {
        assert_eq!(vec![15, 28, 32, 48, 54],
                   compute_score(vec![(5, 5), (5, 5), (3, 1), (5, 5), (6, 0)]));
    }

    #[test]
    fn test_short_game_ending_in_spare() {
        assert_eq!(vec![10],
                   compute_score(vec![(5, 5)]));
    }

    #[test]
    fn test_game_ending_in_spare() {
        assert_eq!(vec![5, 7, 17],
                   compute_score(vec![(3, 2), (1, 1), (5, 5)]));
    }

    #[test]
    fn test_game_ending_in_strike() {
        assert_eq!(vec![3, 10, 20],
                   compute_score(vec![(1, 2), (3, 4), (10, 0)]))
    }

    #[test]
    fn test_spare_strike() {
        assert_eq!(vec![20, 34, 38],
                   compute_score(vec![(3, 7), (10, 0), (3, 1)]))
    }

    #[test]
    fn test_spare_ending_in_strike() {
        assert_eq!(vec![9, 18, 23, 35, 55, 65],
                   compute_score(vec![(1, 8), (3, 6), (3, 2), (7, 3), (2, 8), (10, 0)]))
    }

    #[test]
    fn test_spare_strike_then_some() {
        assert_eq!(vec![20, 33, 36, 43, 46],
                   compute_score(vec![(3, 7), (10, 0), (1, 2), (3, 4), (1, 2)]))
    }

    #[test]
    fn test_simple_strike_two_frames_left() {
        assert_eq!(vec![13, 16, 23],
                   compute_score(vec![(10, 0), (1, 2), (3, 4)]))
    }

    #[test]
    fn test_strike_one_frame_left() {
        assert_eq!(vec![13, 16],
                   compute_score(vec![(10, 0), (1, 2)]))
    }

    #[test]
    fn test_strike_no_frames_left() {
        assert_eq!(vec![10],
                   compute_score(vec![(10, 0)]))
    }

    #[test]
    fn test_max_score() {
        assert_eq!(vec![30, 60, 90, 120, 150, 180, 210, 240, 270, 300],
                   compute_score(vec![(10, 0), (10, 0), (10, 0), (10, 0), (10, 0), (10, 0), (10, 0), (10, 0), (10, 0), (10, 0), (10, 10)]))
    }
}