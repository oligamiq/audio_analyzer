use fundsp::hacker32::*;
use plotters::prelude::*;

use anyhow::Result;

use crate::console::TextDrawingBackend;

pub fn plot() -> Result<()> {
    draw_chart(TextDrawingBackend::new().into_drawing_area())?;

    let b = BitMapBackend::new("image.png", (1280, 640)).into_drawing_area();
    b.fill(&WHITE)?;
    draw_chart(b)?;

    Ok(())
}

// pub fn draw_chart<DB: DrawingBackend>(b: DrawingArea<DB, plotters::coord::Shift>) -> Result<()>
// where
//     DB::ErrorType: 'static,
// {
//     let mut chart = ChartBuilder::on(&b)
//         .caption("image, 8 octaves", ("sans-serif", 40).into_font())
//         .margin(5)
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         .build_cartesian_2d(-2f32..2f32, -1f32..1f32)?;

//     chart.configure_mesh().draw()?;

//     chart
//         .draw_series(LineSeries::new(
//             // (-200..=200)
//             //     .map(|x| x as f32 / 100.0)
//             //     .map(|x| (x, fractal_noise(5, 8, 0.3, x))),
//             (-200..=200).map(|x| (x as f32 / 100.0, x as f32 / 200.0)),
//             RGBColor(0, 64, 192).stroke_width(2),
//         ))?
//         .label("roughness 0.3")
//         .legend(|(x, y)| {
//             PathElement::new(
//                 vec![(x, y), (x + 20, y)],
//                 RGBColor(0, 64, 192).stroke_width(2),
//             )
//         });

//     chart
//         .configure_series_labels()
//         .background_style(WHITE.mix(0.8))
//         .border_style(BLACK)
//         .label_font(("Calibri", 20))
//         .draw()?;

//     b.present()?;

//     Ok(())
// }

pub fn draw_chart<DB: DrawingBackend>(b: DrawingArea<DB, plotters::coord::Shift>) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let mut chart = ChartBuilder::on(&b)
        .caption("image, 8 octaves", ("sans-serif", 1).into_font())
        .margin(4)
        .x_label_area_size(4)
        .y_label_area_size(2)
        .build_cartesian_2d(-2f32..2f32, -1f32..1f32)?;

    // chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (-200..=200).map(|x| (x as f32 / 100.0, x as f32 / 200.0)),
            RGBColor(0, 64, 192).stroke_width(2),
        ))?
        .label("roughness 0.3")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 2, y)],
                RGBColor(0, 64, 192).stroke_width(1),
            )
        });

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font(("Calibri", 20))
        .draw()?;

    b.present()?;

    Ok(())
}
