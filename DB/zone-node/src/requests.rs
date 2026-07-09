use crate::{metadata::Direction, network::_quic_send};
use quinn::Connection;

pub async fn get_neighbor_direction (
    incoming_connection_request: Connection
) -> Result<Direction, String> {

    let message = "Direction";

    let response = _quic_send(
        &incoming_connection_request,
        message
    )
    .await
    .map_err(|e| e.to_string())?;


    let response_direction = match response.as_str() {
        "NORTH" => Direction::North,
        "SOUTH" => Direction::South,
        "EAST" => Direction::East,
        "WEST" => Direction::West,
        _ => return Err("invalid direction".to_string()),
    };

    let inverse_direction = response_direction.inverse();

    Ok(inverse_direction)
}