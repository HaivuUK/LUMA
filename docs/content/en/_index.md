---
date: '2026-06-12T15:54:45+01:00'
title: 'LUMA'
description: 'Local Unit Modulus Assignment. A modern high-performance rust tool for computed tomography derived material property assignment for finite element analysis.'
layout: hextra-home
---
<section class="hero hx-py-40 hx-text-center" id="hero"> 

  <div class="hero-fem" data-rows="32">
    <svg class="fem" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 24 150 28" preserveAspectRatio="none" shape-rendering="auto"></svg>
  </div>

  {{< hextra/hero-badge link="https://github.com/HaivuUK/luma" >}}<div class="hx-w-2 hx-h-2 hx-rounded-full hx-bg-primary-400"></div>
  <span>Releasing Autumn 2026</span>
  {{< icon name="arrow-circle-right" attributes="height=14" >}}{{< /hextra/hero-badge >}}

  <!-- <div class="hx-mt-6 hx-mb-4">
    <h1 class="not-prose hx-text-4xl md:hx-text-9xl hx-font-bold hx-leading-none hx-tracking-tighter hx-py-2 hx-bg-clip-text">L U M A</h1>
  </div> -->

  <div class="not-prose hx-mt-8 hx-mb-6 hx-w-full hx-px-4">
    <div class="hx-flex hx-flex-row hx-justify-center hx-items-start hx-w-full hx-max-w-6xl hx-mx-auto">
      <div class="hx-text-center" style="width: 10%;"></div>
      <div class="hx-text-center" style="width: 20%;">
        <span class="hx-block hx-font-bold hx-leading-none hx-py-2 hx-bg-clip-text" 
              style="font-size: clamp(2.5rem, 13vw, 8.5rem);">L</span>
        <span class="hx-block hx-font-medium hx-text-gray-700 dark:hx-text-gray-200 hx-tracking-wide hx-mt-2" 
              style="font-size: clamp(0.5rem, 2.2vw, 1.25rem);">Local</span>
      </div>
      <div class="hx-text-center" style="width: 20%;">
        <span class="hx-block hx-font-bold hx-leading-none hx-py-2 hx-bg-clip-text" 
              style="font-size: clamp(2.5rem, 13vw, 8.5rem);">U</span>
        <span class="hx-block hx-font-medium hx-text-gray-700 dark:hx-text-gray-200 hx-tracking-wide hx-mt-2" 
              style="font-size: clamp(0.5rem, 2.2vw, 1.25rem);">Unit</span>
      </div>
      <div class="hx-text-center" style="width: 20%;">
        <span class="hx-block hx-font-bold hx-leading-none hx-py-2 hx-bg-clip-text" 
              style="font-size: clamp(2.5rem, 13vw, 8.5rem);">M</span>
        <span class="hx-block hx-font-medium hx-text-gray-700 dark:hx-text-gray-200 hx-tracking-wide hx-mt-2" 
              style="font-size: clamp(0.5rem, 2.2vw, 1.25rem);">Modulus</span>
      </div>
      <div class="hx-text-center" style="width: 20%;">
        <span class="hx-block hx-font-bold hx-leading-none hx-py-2 hx-bg-clip-text" 
              style="font-size: clamp(2.5rem, 13vw, 8.5rem);">A</span>
        <span class="hx-block hx-font-medium hx-text-gray-700 dark:hx-text-gray-200 hx-tracking-wide hx-mt-2" 
              style="font-size: clamp(0.5rem, 2.2vw, 1.25rem);">Assignment</span>
      </div>
      <div class="hx-text-center" style="width: 10%;"></div>
    </div>
  </div>

  <div class="hx-mb-6">
    <p class="hx-text-lg">A modern high-performance rust tool for computed tomography derived material property assignment for finite element analysis.</p>
  </div>

  <a href="docs" class="not-prose hx-font-medium hx-cursor-pointer hx-select-none hx-w-64 hx-px-10 hx-py-4 hx-rounded-lg hx-text-center hx-text-white hx-inline-block hx-bg-primary-600 hover:hx-bg-primary-700 dark:hx-bg-primary-600 dark:hover:hx-bg-primary-700 hx-transition-all hx-ease-in hx-duration-200 start-now-button" style="margin: 2px; align-items: center; justify-content: center; display: inline-flex;">
    {{< icon "document-text" >}} &nbsp; Get Started 
  </a>

  <a href="https://github.com/HaivuUK/luma" class="not-prose hx-font-medium hx-cursor-pointer hx-select-none hx-w-64 hx-px-10 hx-py-4 hx-rounded-lg hx-text-center hx-text-white hx-inline-block hx-bg-primary-600 hover:hx-bg-primary-700 dark:hx-bg-primary-600 dark:hover:hx-bg-primary-700 hx-transition-all hx-ease-in hx-duration-200 start-now-button" style="margin: 2px; align-items: center; justify-content: center; display: inline-flex;">
    {{< icon "github" >}} &nbsp; Code 
  </a>

  <a class="not-prose hx-font-medium hx-cursor-pointer hx-select-none hx-w-64 hx-px-10 hx-py-4 hx-rounded-lg hx-text-center hx-text-black dark:hx-text-white hx-inline-block not-sure-button hx-transition-all hx-ease-in hx-duration-200" style="margin: 2px;" onclick="scrollDownSection();">Why use LUMA?</a>

</section>

<section id="what-is-luma">
  <h2 class="hx-text-4xl hx-font-bold md:hx-text-6xl hx-inline">What is LUMA</h2>
  <div class="hover-cards-grid">
    <div class="hover-card">
      <h2 class="hx-text-2xl hx-font-bold">LUMA is Fast</h2>
      <p class="hx-text-base">Built in rust LUMA is designed for high performance, ensuring quick and efficient processing of complex finite element models no matter how many materials you want. LUMA has also been tested to work with $\mu$CT data.</p>
    </div>
    <div class="hover-card">
      <h2 class="hx-text-2xl hx-font-bold">Focused on User Experience</h2>
      <p class="hx-text-base">LUMA comes with some quality of life features that limit your need to jump between different tools. Such as histogram reports, CT calibration, visualisation, and alignment all in one place.</p>
    </div>
    <div class="hover-card">
      <h2 class="hx-text-2xl hx-font-bold">Open Source</h2>
      <p class="hx-text-base">LUMA is open source, allowing the community to contribute and improve the software together.</p>
    </div>
    <div class="hover-card">
      <h2 class="hx-text-2xl hx-font-bold">CLI First</h2>
      <p class="hx-text-base">LUMA is designed with a command-line interface first approach, making it easy to integrate into automated workflows and scripts, or batch processes.</p>
    </div>
    <div class="hover-card cta-card">
      <h2 class="hx-text-2xl hx-font-bold">Ready to map some bones?</h2>
      <p class="hx-text-base">Download the latest version of LUMA and get started.</p>
      {{< hextra/hero-button text="Read Our Quickstart Guide" link="docs" style="margin-top: 15px; border-radius: 30px;" >}}
    </div>
  </div>
</section>

<section id="explore">
  <h2 class="hx-text-4xl hx-font-bold md:hx-text-6xl hx-text-center">
    Explore the documentation or visit the repository
  </h2>
  <p class="hx-text-base hx-text-center hx-mb-10px">
    Take a look around and see what we have planned and the documentation we have already.
  </p>
  <div class="hover-cards-grid">
    <a href="docs" class="hover-card">
      <h2 class="hx-text-2xl hx-inline-flex hx-font-bold">{{< icon name="newspaper" attributes="height=30" >}}<span>Documentation</span></h2>
      <p>
        Documentation and helpful extras for using LUMA.
      </p>
      <span class="explore-link">Read documentation →</span>
    </a>
    <a href="roadmap" class="hover-card">
      <h2 class="hx-text-2xl hx-inline-flex hx-font-bold">{{< icon name="map" attributes="height=30" >}}<span>Roadmap</span></h2>
      <p>
        Our plans for the future of LUMA.
      </p>
      <span class="explore-link">View roadmap →</span>
    </a>
    <a href="https://github.com/HaivuUK/luma/issues" class="hover-card">
      <h2 class="hx-text-2xl hx-inline-flex hx-font-bold">{{< icon name="inbox-in" attributes="height=30" >}}<span>Report Issues</span></h2>
      <p>
        Found a bug or have a suggestion? Let us know!
      </p>
      <span class="explore-link">Report issues →</span>
    </a>
    <a href="https://github.com/HaivuUK/luma/releases/latest" class="hover-card">
      <h2 class="hx-text-2xl hx-inline-flex hx-font-bold">{{< icon name="download" attributes="height=30" >}}<span>Download</span></h2>
      <p>
        Get the latest version of LUMA.
      </p>
      <span class="explore-link">Download now →</span>
    </a>
    <a href="https://github.com/INSIGNEO/" class="hover-card">
      <h2 class="hx-text-2xl hx-inline-flex hx-font-bold">{{< icon name="github" attributes="height=30" >}}<span>Visit INSIGNEO</span></h2>
      <p>
        Visit the INSIGNEO GitHub to see what other projects people in the group have been working on.
      </p>
      <span class="explore-link">Visit INSIGNEO →</span>
    </a>
  </div>
</section>

<section id="contributors" class="hx-mb-16">
    <h2 class="hx-text-4xl hx-font-bold md:hx-text-6xl">We Need Your Help!</h2>
    <p class="hx-text-base">LUMA is 100% free and will always remain so! However, it relies on contributors and the community to thrive.<br>Here are some ways you can help:</p>
    <div style="display: inline-flex; flex-wrap: wrap; justify-content: center; margin-top: 2rem;">
        {{< hextra/hero-button text="Contribute" link="about/write-content" icon="pencil" class="contributors-button" >}}
        {{< hextra/hero-button text="Translate" link="about/translate" icon="translate" class="contributors-button" >}}
        {{< hextra/hero-button text="Validate" link="about/validation" icon="badge-check" class="contributors-button" >}}
        <!-- {{< hextra/hero-button text="Donate" link="about/donate" icon="heart" class="contributors-button" >}} -->
        {{< hextra/hero-button text="Share" onclick="toggleShareDropdown();" icon="share" class="contributors-button shareDropdownButton" noLink="true" >}}
        <div id="shareDropdown" class="dropdown-content">
          <a href="https://www.reddit.com/submit?url=https%3A%2F%2Fgithub.com%2FHaivuUK%2Fluma&title=Check%20out%20this%20modern%20finite%20element%20method%20material%20mapping%20software%20built%20in%20rust." target="_blank" rel="noopener noreferrer" style="display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 20px; color: #fff; background-color: #FF4500; text-decoration: none; border-radius: 5px; font-family: sans-serif; font-weight: bold; font-size: 1rem;">
<svg width="24" height="24" fill="#fff" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M14.5 15.41C14.58 15.5 14.58 15.69 14.5 15.8C13.77 16.5 12.41 16.56 12 16.56C11.61 16.56 10.25 16.5 9.54 15.8C9.44 15.69 9.44 15.5 9.54 15.41C9.65 15.31 9.82 15.31 9.92 15.41C10.38 15.87 11.33 16 12 16C12.69 16 13.66 15.87 14.1 15.41C14.21 15.31 14.38 15.31 14.5 15.41M10.75 13.04C10.75 12.47 10.28 12 9.71 12C9.14 12 8.67 12.47 8.67 13.04C8.67 13.61 9.14 14.09 9.71 14.08C10.28 14.08 10.75 13.61 10.75 13.04M14.29 12C13.72 12 13.25 12.5 13.25 13.05S13.72 14.09 14.29 14.09C14.86 14.09 15.33 13.61 15.33 13.05C15.33 12.5 14.86 12 14.29 12M22 12C22 17.5 17.5 22 12 22S2 17.5 2 12C2 6.5 6.5 2 12 2S22 6.5 22 12M18.67 12C18.67 11.19 18 10.54 17.22 10.54C16.82 10.54 16.46 10.7 16.2 10.95C15.2 10.23 13.83 9.77 12.3 9.71L12.97 6.58L15.14 7.05C15.16 7.6 15.62 8.04 16.18 8.04C16.75 8.04 17.22 7.57 17.22 7C17.22 6.43 16.75 5.96 16.18 5.96C15.77 5.96 15.41 6.2 15.25 6.55L12.82 6.03C12.75 6 12.68 6.03 12.63 6.07C12.57 6.11 12.54 6.17 12.53 6.24L11.79 9.72C10.24 9.77 8.84 10.23 7.82 10.96C7.56 10.71 7.2 10.56 6.81 10.56C6 10.56 5.35 11.21 5.35 12C5.35 12.61 5.71 13.11 6.21 13.34C6.19 13.5 6.18 13.62 6.18 13.78C6.18 16 8.79 17.85 12 17.85C15.23 17.85 17.85 16.03 17.85 13.78C17.85 13.64 17.84 13.5 17.81 13.34C18.31 13.11 18.67 12.6 18.67 12Z" /></svg>
<span>Share on Reddit</span>
</a>
          <a href="https://www.facebook.com/sharer/sharer.php?u=https%3A%2F%2Fgithub.com%2FHaivuUK%2Fluma" target="_blank" rel="noopener noreferrer" style="display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 20px; color: #fff; background-color: #1877F2; text-decoration: none; border-radius: 5px; font-family: sans-serif; font-weight: bold; font-size: 1rem;">
<svg width="24" height="24" fill="#fff" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2.04C6.5 2.04 2 6.53 2 12.06C2 17.06 5.66 21.21 10.44 21.96V14.96H7.9V12.06H10.44V9.85C10.44 7.34 11.93 5.96 14.22 5.96C15.31 5.96 16.45 6.15 16.45 6.15V8.62H15.19C13.95 8.62 13.56 9.39 13.56 10.18V12.06H16.34L15.89 14.96H13.56V21.96A10 10 0 0 0 22 12.06C22 6.53 17.5 2.04 12 2.04Z" /></svg>
<span>Share on Facebook</span>
</a>
          <a href="https://twitter.com/intent/tweet?url=https%3A%2F%2Fgithub.com%2FHaivuUK%2Fluma&text=Check%20out%20this%20modern%20finite%20element%20method%20material%20mapping%20software%20built%20in%20rust." target="_blank" rel="noopener noreferrer" style="display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 20px; color: #fff; background-color: #000000; text-decoration: none; border-radius: 5px; font-family: sans-serif; font-weight: bold; font-size: 1rem;">
<svg width="24" height="24" fill="#fff" viewBox="0 0 24 24"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>
<span>Share on X (Twitter)</span>
</a>
          <a href="https://www.linkedin.com/sharing/share-offsite/?url=https%3A%2F%2Fgithub.com%2FHaivuUK%2Fluma" target="_blank" rel="noopener noreferrer" style="display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 20px; color: #fff; background-color: #0A66C2; text-decoration: none; border-radius: 5px; font-family: sans-serif; font-weight: bold; font-size: 1rem;">
<svg width="24" height="24" fill="#fff" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M19 3A2 2 0 0 1 21 5V19A2 2 0 0 1 19 21H5A2 2 0 0 1 3 19V5A2 2 0 0 1 5 3H19M18.5 18.5V13.2A3.26 3.26 0 0 0 15.24 9.94C14.39 9.94 13.4 10.46 12.92 11.24V10.13H10.13V18.5H12.92V13.57C12.92 12.8 13.54 12.17 14.31 12.17A1.4 1.4 0 0 1 15.71 13.57V18.5H18.5M6.88 8.56A1.68 1.68 0 0 0 8.56 6.88C8.56 5.95 7.81 5.19 6.88 5.19A1.69 1.69 0 0 0 5.19 6.88C5.19 7.81 5.95 8.56 6.88 8.56M8.27 18.5V10.13H5.5V18.5H8.27Z" /></svg>
<span>Share on LinkedIn</span>
</a>
        </div>
    </div>
    <a href="https://github.com/HaivuUK/luma/graphs/contributors"><p class="hx-text-base hx-underline hx-mt-4">View all contributors</p></a>
</section>
