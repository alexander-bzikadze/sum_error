# Sum error
The library is ment to ease coding functions with try calls (or with the ? operator) and provides a derive macro to easily sum errors into a enum that automaticaly derives all the required traits including [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html) and [std::convert::From](https://doc.rust-lang.org/std/convert/trait.From.html) from all the contained error types.

In order to use this functionality create a enum containing unnamed variants with a single error type. 
Then use the derive macro. 
And now it is done!

# Example
To better illustrate described functionality consider the following example.
```rust
use std::fs::File;
use std::rc::Rc;
use sprite::Sprite;
use piston_window::Texture;
use image::gif::Decoder;
use image::AnimationDecoder;
use piston_window::{TextureContext, TextureSettings};
use sum_error::*;

/// Load a gif sprite.
pub fn load<F: gfx::Factory<R>,
            R: gfx::Resources,
            C: gfx::CommandBuffer<R>>(ctx: &mut TextureContext<F, R, C>)
                -> Result<Vec<Sprite<Texture<R>>>, CombineError> {
    let file = File::open("file.gif")?;
    let decoder = Decoder::new(file)?;
    let frames = decoder.into_frames().collect_frames()?;
    frames.iter()
        .map(|frame| {
            Texture::from_image(ctx, frame.buffer(), &TextureSettings::new())
                .map(Rc::new)
                .map(Sprite::from_texture)
                .map_err(|e| e.into())
        }).collect()
}

#[derive(SumError)]
pub enum CombineError {
    FileError(std::io::Error),
    ImageError(image::ImageError),
    GfxError(gfx_texture::Error),
}
```
Using SumError derive macro allows us to effectively describe summing error type with standard tools and then don't waste precious time on writing tons of traits' implementations in order to use it conviniently.

# Requirments for the enum
* First of all, it should be a enum -- not a struct, not a union. Enum
* Variants should contain only one unnamed field
* Types of the fields should implement [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)
* Types of the fields should be unique inside the enum scope (as otherwise [std::convert::From](https://doc.rust-lang.org/std/convert/trait.From.html) trait would be impossible to imlement)

# Alternatives
If you do not need the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html) functionality for your summing class, consider using [sum_type](https://docs.rs/sum_type/0.2.0/sum_type/) crate. 
It does provide functionality to easily sum types and implement [std::convert::From](https://doc.rust-lang.org/std/convert/trait.From.html) traits.
However, beware as it does not use cute derive syntax but raw macro by design.

# Contribution
Feel free to raise issues, demand more functionality and propose any enhancements. You can use github tools for that or address the authors directly via email.

# Authors
* [Alexander Bzikadze](mailto:alexander.bzikadze@gmail.com)
