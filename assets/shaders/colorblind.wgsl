#import bevy_pbr::mesh_view_bind_group


// translated from https://github.com/electronicarts/Tunable-Colorblindness-Solution

// Description: Accessibility support library for shaders, covering brightness, contrast, and colorblindness issues.
//
// Copyright (c) 2015-2021 Electronic Arts Inc. 
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//*** Variables ************************************************************************************/

let colorBlindProtanopiaFactor = 0.0; // pass in 0 or 1 to turn on support
let colorBlindDeuteranopiaFactor = 0.0; // pass in 0 or 1 to turn on support
let colorBlindTritanopiaFactor: f32 = 0.0; // pass in 0 or 1 to turn on support

let colorBlindDaltonizeFactor: f32 = 0.0; // pass in 0 or 0.9 for best results
let accessibilityBrightnessFactor: f32 = 0.0; // zero is no effect
let accessibilityContrastFactor: f32 = 0.0; // zero is no effect

// suggested brightness factors: -0.1, -0.05, 0, 0.05, 0.11
// suggested contrast factors: -0.25, -0.12, 0.2, 0.4

//*** Methods ************************************************************************************/


// Shifts from rgb to luminosity color representation. The magic numbers
// are standard conversion values used to do this.
// see https://en.wikipedia.org/wiki/CIE_1931_color_space for details
fn RgbToLms(color: vec3<f32>) -> vec3<f32>
{
    var l = (17.8824 * color.r) + (43.5161 * color.g) + (4.11935 * color.b);
    var m = (3.45565 * color.r) + (27.1554 * color.g) + (3.86714 * color.b);
    var s = (0.0299566 * color.r) + (0.184309 * color.g) + (1.46709 * color.b);
    return vec3<f32>(l,m,s);   
}

// Shifts from luminosity to rgb color representation. The magic numbers
// are standard conversion values used to do this.
// see https://en.wikipedia.org/wiki/LMS_color_space for details
fn LmsToRgb(color: vec3<f32>) -> vec3<f32>
{
    var r = (0.0809444479 * color.r) + (-0.130504409 * color.g) + (0.116721066 * color.b);
    var g = (-0.0102485335 * color.r) + (0.0540193266 * color.g) + (-0.113614708 * color.b);
    var b = (-0.000365296938 * color.r) + (-0.00412161469 * color.g) + (0.693511405 * color.b);
    return vec3<f32>(r,g,b);
}

// Shifts colors based on color blind color weaknesses to areas where user can better see.
// The magic numbers model the way the human eye works when affected by different color
// deficiencies. They will never change.
// see http://www.daltonize.org/search/label/Color%20Blindness for details
fn Daltonize(color: vec4<f32>) -> vec4<f32>
{
    var colorLMS: vec3<f32> = color.rgb;
    colorLMS = RgbToLms(colorLMS);
    
    var colorWeak: vec3<f32>;
    
    colorWeak.r = (2.02344*colorLMS.g - 2.5281*colorLMS.b)*colorBlindProtanopiaFactor + colorLMS.r*(1.0-colorBlindProtanopiaFactor);
    colorWeak.g = (0.494207*colorLMS.r + 1.24827*colorLMS.b)*colorBlindDeuteranopiaFactor + colorLMS.g*(1.0-colorBlindDeuteranopiaFactor);
    colorWeak.b = (-0.395913*colorLMS.r + 0.801109*colorLMS.g)*colorBlindTritanopiaFactor + colorLMS.b*(1.0-colorBlindTritanopiaFactor);
    
    colorWeak = LmsToRgb(colorWeak);
    
    colorWeak = color.rgb - colorWeak;
    
    var colorShift: vec3<f32>;
    colorShift.r = 0.0;
    colorShift.g = colorWeak.g + 0.7*colorWeak.r;
    colorShift.b = colorWeak.b + 0.7*colorWeak.r;
    
    var finalColor = color;
    finalColor.r = finalColor.r + colorShift.r;
    finalColor.g = finalColor.g + colorShift.g;
    finalColor.b = finalColor.b + colorShift.b;
    
    //finalColor = clamp(finalColor,0.0,1.0);
    finalColor.r = clamp(finalColor.r,0.0,1.0);
    finalColor.g = clamp(finalColor.g,0.0,1.0);
    finalColor.b = clamp(finalColor.b,0.0,1.0);
    
    return finalColor;
}

// CALL THIS METHOD TO PROCESS COLOR
// applies brightness, contrast, and color blind settings to passed in color
fn AccessibilityPostProcessing(color: vec4<f32>) -> vec4<f32>
{
    var finalColor = color;
    // apply contrast shift for daltonization
    //finalColor.rgb = ((finalColor.rgb - 0.5) * (1.0+colorBlindDaltonizeFactor*0.112)) + 0.5;
    finalColor.r = ((finalColor.r - 0.5) * (1.0+colorBlindDaltonizeFactor*0.112)) + 0.5;
    finalColor.g = ((finalColor.g - 0.5) * (1.0+colorBlindDaltonizeFactor*0.112)) + 0.5;
    finalColor.b = ((finalColor.b - 0.5) * (1.0+colorBlindDaltonizeFactor*0.112)) + 0.5;

    // apply brightness shift for daltonization
    //finalColor.rgb -= 0.075*colorBlindDaltonizeFactor;
    finalColor.r = finalColor.r - 0.075*colorBlindDaltonizeFactor;
    finalColor.g = finalColor.g - 0.075*colorBlindDaltonizeFactor;
    finalColor.b = finalColor.b - 0.075*colorBlindDaltonizeFactor;
    
    // apply colorblind compensation algorithm
    finalColor = (Daltonize(finalColor)*colorBlindDaltonizeFactor + finalColor*(1.0-colorBlindDaltonizeFactor));
    
    // expose contrast
    //finalColor.rgb = ((finalColor.rgb - 0.5) * (1.0+accessibilityContrastFactor)) + 0.5;
    finalColor.r = ((finalColor.r - 0.5) * (1.0+accessibilityContrastFactor)) + 0.5;
    finalColor.g = ((finalColor.g - 0.5) * (1.0+accessibilityContrastFactor)) + 0.5;
    finalColor.b = ((finalColor.b - 0.5) * (1.0+accessibilityContrastFactor)) + 0.5;

    // expose brightness & shift colors back to lighter hues
    //finalColor.rgb += accessibilityBrightnessFactor+0.08*colorBlindDaltonizeFactor;
    finalColor.r = finalColor.r + accessibilityBrightnessFactor+0.08*colorBlindDaltonizeFactor;
    finalColor.g = finalColor.g + accessibilityBrightnessFactor+0.08*colorBlindDaltonizeFactor;
    finalColor.b = finalColor.b + accessibilityBrightnessFactor+0.08*colorBlindDaltonizeFactor;
    
    return finalColor;
}

[[group(1), binding(0)]]
var texture: texture_2d<f32>;

[[group(1), binding(1)]]
var our_sampler: sampler;

[[stage(fragment)]]
fn fragment([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    // Get screen position with coordinates from 0 to 1
    let uv = position.xy / vec2<f32>(view.width, view.height);
    let offset_strength = 0.02;

    // Sample each color channel with an arbitrary shift
    var output_color = vec4<f32>(
        textureSample(texture, our_sampler, uv + vec2<f32>(offset_strength, -offset_strength)).r,
        textureSample(texture, our_sampler, uv + vec2<f32>(-offset_strength, 0.0)).g,
        textureSample(texture, our_sampler, uv + vec2<f32>(0.0, offset_strength)).b,
        1.0
        );
    output_color = AccessibilityPostProcessing(output_color);
    return output_color;
}
