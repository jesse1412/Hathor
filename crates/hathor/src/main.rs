use yew::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Clone)]
struct TrackRecord { 
    id: String,
    image: String,
}

#[function_component(App)]
pub fn app() -> Html {
    let mut tracks: Vec<TrackRecord> = vec![
        TrackRecord { 
            id: "Track1".to_string(),
            image: "https://avatars.githubusercontent.com/u/15838364?v=4".to_string(),
        },
    ];


    for _ in 1..5 { 
        tracks.push(tracks.get(0).unwrap().clone());  
    }



    let tracks = tracks.iter().map(|track| html!{
        <img class="image" src={format!("{}",track.image)} draggable="false"/>
    }).collect::<Html>();

    html! {
        <div>
        <div id="image-track" data-mouse-down-at="0" data-prev-percentage="0">
        { tracks }
        </div>
            <MediaControl/>
        </div>
    }
}


#[function_component]
fn MediaControl() -> Html {
    html!{ 
        <div id="bottom-bar"> 
        <button class="media-control-button"> {"P"} </button>
        <button class="media-control-button"> {"P/P"} </button>
        <button class="media-control-button"> {"R"} </button>
        <div id="song-time-track">
            <span style="width: 0px ;"> </span>
        </div>
        </div>
    }
}

