use tiberius::{Client, Query};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;
use crate::parser::ChartInfo;

pub async fn upload_charts (
    client: &mut Client<Compat<TcpStream>>,
    charts: &Vec<ChartInfo>,
) -> Result<(), tiberius::error::Error> {
    for chart in charts {
        // 기존 데이터 삭제 #1
        let mut query = Query::new("DELETE FROM dbo.o2jam_music_metadata WHERE MusicCode = @P1");
        query.bind(chart.chart_id);
        query.execute(client).await?;

        // 기존 데이터 삭제 #2
        let mut query = Query::new("DELETE FROM dbo.o2jam_music_data WHERE MusicCode = @P1");
        query.bind(chart.chart_id);
        query.execute(client).await?;

        // 메타데이터 추가
        let mut query = Query::new("INSERT INTO dbo.o2jam_music_metadata VALUES(@P1, @P2, @P3, @P4, @P5)");
        query.bind(chart.chart_id);
        query.bind(&chart.title);
        query.bind(&chart.artist);
        query.bind(&chart.chart_maker);
        query.bind(chart.bpm);
        query.execute(client).await?;

        // 데이터 추가
        for i in 0..3 {
            let mut query = Query::new("INSERT INTO dbo.o2jam_music_data VALUES(@P1, @P2, @P3, @P4, 0)");
            query.bind(chart.chart_id);
            query.bind(i as i32);
            query.bind(chart.note_count[i]);
            query.bind(chart.level[i]);
            query.execute(client).await?;
        }

        println!("Uploaded: {}", chart.to_string());
    }

    Ok(())
}

pub async fn update_charts (
    client: &mut Client<Compat<TcpStream>>,
    charts: &Vec<ChartInfo>,
) -> Result<(), tiberius::error::Error> {
    for chart in charts {
        // 메타데이터 수정
        let mut query = Query::new("UPDATE dbo.o2jam_music_metadata SET Title = @P2, Artist = @P3, NoteCharter = @P4 WHERE MusicCode = @P1");
        query.bind(chart.chart_id);
        query.bind(&chart.title);
        query.bind(&chart.artist);
        query.bind(&chart.chart_maker);
        query.execute(client).await?;

        // 데이터 수정
        for i in 0..3 {
            let mut query = Query::new("UPDATE dbo.o2jam_music_data SET NoteLevel = @P3, NoteCount = @P4 WHERE MusicCode = @P1 AND Difficulty = @P2");
            query.bind(chart.chart_id);
            query.bind(i as i32);
            query.bind(chart.level[i]);
            query.bind(chart.note_count[i]);
            query.execute(client).await?;
        }

        println!("Updated: {}", chart.to_string());
    }

    Ok(())
}

pub async fn delete_charts (
    client: &mut Client<Compat<TcpStream>>,
    charts: &Vec<ChartInfo>,
) -> Result<(), tiberius::error::Error> {
    for chart in charts {
        // 기존 데이터 삭제 #1
        let mut query = Query::new("DELETE FROM dbo.o2jam_music_metadata WHERE MusicCode = @P1");
        query.bind(chart.chart_id);
        query.execute(client).await?;

        // 기존 데이터 삭제 #2
        let mut query = Query::new("DELETE FROM dbo.o2jam_music_data WHERE MusicCode = @P1");
        query.bind(chart.chart_id);
        query.execute(client).await?;

        println!("Deleted: {}", chart.to_string());
    }

    Ok(())
}


