<div id="top"></div>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->

[![Latest release][release-shield]][release-url]
[![Release build][build-shield]][build-url]
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]

<div align="center">
  <a href="https://linkedin.com/in/gabriel-guldner">
  <img src="https://img.shields.io/badge/-LinkedIn-black.svg?style=social&logo=linkedin&colorB=555" />
  </a>
</div>

<!-- PROJECT LOGO -->
<br />
<div align="center">
<h3 align="center">Simple Repository Manager</h3>

  <p align="center">
    A very simple, lightweight repository manager written in rust.
    <br />
    <br />
    <a href="https://github.com/dasbaumwolltier/simple-repository-manager/issues">Report Bug</a>
    ·
    <a href="https://github.com/dasbaumwolltier/simple-repository-manager/issues">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#docker">Docker</a></li>
        <li><a href="#building">Building</a></li>
        <ul>
          <li><a href="#prerequisites">Prerequisites</a></li>
          <li><a href="#installation">Installation</a></li>
        </ul>
      </ul>
    </li>
    <li><a href="#usage">Configuration</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

This project was started becasue I wanted something like [Nexus OSS](https://github.com/sonatype/nexus-public) to host my Archlinux repository. I, however, wanted something lightweight, as it should not eat resources doing nothing most of the time.

As such I first implemented a raw `FileRepository`, which can just `GET` and `PUT` arbitrary files inside the repository, storing them on disk. 

<p align="right">(<a href="#top">back to top</a>)</p>



### Built With

* [Rust](https://www.rust-lang.org/)
* [Rocket](https://rocket.rs/)
* [Serde](https://serde.rs/)
* Other Packages: See `Cargo.toml`

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

There are two ways to use the server:
1. Docker
2. Build it yourself

### Docker

For this to work you have to have a working docker installation and a `config.yaml` (see [Configuration](#Configuration)) file. The you can run the docker container with:
```shell
docker run -p 8000:8000 -v $(realpath config.yaml):/config.yaml dasbaumwolltier/simple-repository-manager:0.1.4
```

### Building
#### Prerequisites

This software is built with rust, so you need an up to date `stable` rust installation. For this consult the examples below or your favourite search engine.

* Archlinux
```shell
sudo pacman -S rustup
rustup toolchain install stable
```
* Other Linux Distros (with curl installed)
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
```

#### Installation

You can build a binary from the repository by first cloning the repository, then running:

```shell
cargo build --release
```

After which you will find the binary under `target/release/simple-repository-manager`. This binary the can be run like so for example `target/release/simple-repository-manager --config config.yaml`.

<!-- USAGE EXAMPLES -->
## Configuration

The server is configured using a `config.yaml` file, of which all options are documented thoroughly in the `template-config.yaml`.

The only command line options available are:
* `-c` | `--config`: The path of the `config.yaml` file
* `-h` | `--host` (optional): The host the server should listen on
* `-p` | `--port` (optional): The port the server should listen on
* `-v` | `--verbose` (optional, repeatable): Changes the log level from `Info` to `Debug` and `Trace` with `1` and `2` occurrences respectively

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- ROADMAP -->
## Roadmap

- [ ] Cache Header 
  - [ ] ETag/Last Modified
- [ ] Retention Policy
  - [ ] Add `deleteFile` method to provider
  - [ ] Add `DELETE` Endpoint
- [ ] Refactor authentication
  - [ ] Move authentication out of `Repository` and leave authorization in `Repository`
  - [ ] Create `AuthenticationProvider`
  - [ ] (Add more sophisticated user management)
- [ ] Documentation
- [ ] Add more providers
  - [ ] ...

See the [open issues](https://github.com/dasbaumwolltier/simple-repository-manager/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have suggestions that would make the project better, please open an issue with the tag "enhancement". If you want to implement it, please also first open an issue to discuss.  
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- LICENSE -->
## License

This work is licensed the Apache License 2.0. See `LICENSE` for more information.

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

Gabriel Guldner - [@dasbaumwolltier](https://github.com/dasbaumwolltier) - gabriel@guldner.eu

Project Link: [https://github.com/dasbaumwolltier/simple-repository-manager](https://github.com/dasbaumwolltier/simple-repository-manager)

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [Best-README-Template](https://github.com/othneildrew/Best-README-Template)

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[build-shield]: https://img.shields.io/drone/build/Mirrors/simple-repository-manager/release?server=https%3A%2F%2Fdrone.guldner.eu
[build-url]: https://drone.guldner.eu/Mirrors/simple-repository-manager
[release-shield]: https://img.shields.io/github/v/release/dasbaumwolltier/simple-repository-manager?display_name=tag&sort=semver
[release-url]: https://github.com/dasbaumwolltier/simple-repository-manager/releases
[contributors-shield]: https://img.shields.io/github/contributors/dasbaumwolltier/simple-repository-manager.svg?style=flat
[contributors-url]: https://github.com/dasbaumwolltier/simple-repository-manager/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/dasbaumwolltier/simple-repository-manager.svg?style=flat
[forks-url]: https://github.com/dasbaumwolltier/simple-repository-manager/network/members
[stars-shield]: https://img.shields.io/github/stars/dasbaumwolltier/simple-repository-manager.svg?style=flat
[stars-url]: https://github.com/dasbaumwolltier/simple-repository-manager/stargazers
[issues-shield]: https://img.shields.io/github/issues/dasbaumwolltier/simple-repository-manager.svg?style=flat
[issues-url]: https://github.com/dasbaumwolltier/simple-repository-manager/issues
[license-shield]: https://img.shields.io/github/license/dasbaumwolltier/simple-repository-manager.svg?style=flat
[license-url]: https://github.com/dasbaumwolltier/simple-repository-manager/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=social&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/gabriel-guldner
[product-screenshot]: images/screenshot.pnge

