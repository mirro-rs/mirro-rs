#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
mod file;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
mod watch;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
pub use watch::watch_config;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
pub use file::read_config_file;

use std::path::PathBuf;

use crate::{
    cli::{self, ArgConfig, Protocol, SelectionSort, ViewSort},
    tui::view::sort::ExportSort,
    utils::tilde::expand_tilde,
};

#[cfg_attr(test, derive(Default))]
#[derive(Debug)]
pub struct Configuration {
    pub outfile: PathBuf,
    pub export: u16,
    pub filters: Vec<Protocol>,
    pub view: ViewSort,
    pub sort: ExportSort,
    pub country: Vec<String>,
    pub ttl: u16,
    pub url: String,
    pub completion_percent: u8,
    pub age: u16,
    pub rate: bool,
    pub connection_timeout: Option<u64>,
    pub include: Option<Vec<String>>,
    pub direct: bool,
}

impl Configuration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        outfile: PathBuf,
        export: u16,
        mut filters: Vec<Protocol>,
        view: ViewSort,
        sort: SelectionSort,
        country: Vec<String>,
        ttl: u16,
        url: String,
        ipv4: bool,
        isos: bool,
        ipv6: bool,
        completion_percent: u8,
        age: u16,
        rate: bool,
        connection_timeout: Option<u64>,
        include: Option<Vec<String>>,
        direct: bool,
    ) -> Self {
        if ipv4 {
            filters.push(Protocol::Ipv4)
        }
        if ipv6 {
            filters.push(Protocol::Ipv6)
        }
        if isos {
            filters.push(Protocol::Isos)
        }

        #[cfg(not(target_os = "windows"))]
        let outfile = expand_tilde(&outfile).expect("home dir to be available");

        Self {
            outfile,
            export,
            filters,
            view,
            sort: match sort {
                SelectionSort::Percentage => ExportSort::Completion,
                SelectionSort::Delay => ExportSort::MirroringDelay,
                SelectionSort::Duration => ExportSort::Duration,
                SelectionSort::Score => ExportSort::Score,
            },
            country,
            ttl,
            url,
            completion_percent,
            age,
            rate,
            connection_timeout,
            include,
            direct,
        }
    }
}

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
fn get_bools(args: &cli::Filters, config: &cli::Filters) -> (bool, bool, bool) {
    let ipv4 = if !args.ipv4 && config.ipv4 {
        true
    } else {
        args.ipv4
    };

    let ipv6 = if !args.ipv6 && config.ipv6 {
        true
    } else {
        args.ipv6
    };

    let isos = if !args.isos && config.isos {
        true
    } else {
        args.isos
    };

    (ipv4, ipv6, isos)
}

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
impl From<(ArgConfig, ArgConfig)> for Configuration {
    fn from((mut args, mut config): (ArgConfig, ArgConfig)) -> Self {
        let (ipv4, isos, ipv6) = get_bools(&args.filters, &config.filters);
        let outfile = args
            .general
            .outfile
            .and_then(expand_tilde)
            .unwrap_or_else(|| config.general.outfile.and_then(expand_tilde).unwrap());
        let export = args
            .general
            .export
            .unwrap_or_else(|| config.general.export.unwrap());
        let filters = args
            .filters
            .protocols
            .unwrap_or_else(|| config.filters.protocols.unwrap());
        let view = args
            .general
            .view
            .unwrap_or_else(|| config.general.view.unwrap());
        let sort = args
            .general
            .sort
            .unwrap_or_else(|| config.general.sort.unwrap());
        let countries = args
            .filters
            .country
            .unwrap_or_else(|| config.filters.country.unwrap());
        let ttl = args
            .general
            .ttl
            .unwrap_or_else(|| config.general.ttl.unwrap());
        let url = args
            .general
            .url
            .unwrap_or_else(|| config.general.url.unwrap());

        let completion = args
            .filters
            .completion_percent
            .unwrap_or_else(|| config.filters.completion_percent.unwrap());

        let age = args
            .filters
            .age
            .unwrap_or_else(|| config.filters.age.unwrap_or_default());

        let rate = if !args.general.rate && config.general.rate {
            true
        } else {
            args.general.rate
        };

        let timoeut = if args.general.timeout.is_none() && config.general.timeout.is_some() {
            config.general.timeout
        } else {
            args.general.timeout
        };

        let include = if args.general.include.is_none() && config.general.include.is_some() {
            std::mem::take(&mut config.general.include)
        } else {
            std::mem::take(&mut args.general.include)
        };
        let direct = if !args.general.direct && config.general.direct {
            true
        } else {
            args.general.direct
        };

        Self::new(
            outfile, export, filters, view, sort, countries, ttl, url, ipv4, isos, ipv6,
            completion, age, rate, timoeut, include, direct,
        )
    }
}

#[cfg(any(test, not(any(feature = "json", feature = "toml", feature = "yaml"))))]
impl From<ArgConfig> for Configuration {
    fn from(args: ArgConfig) -> Self {
        let outfile = args
            .general
            .outfile
            .or_else(|| crate::exit("outfile"))
            .unwrap();
        let export = args.general.export.unwrap_or(cli::DEFAULT_MIRROR_COUNT);
        let filters = args
            .filters
            .protocols
            .unwrap_or_else(|| vec![Protocol::Http, Protocol::Https]);
        let view = args.general.view.unwrap_or_default();
        let sort = args.general.sort.unwrap_or_default();
        let countries = args.filters.country.unwrap_or_default();
        let ttl = args.general.ttl.unwrap_or(cli::DEFAULT_CACHE_TTL);
        let url = args
            .general
            .url
            .unwrap_or_else(|| cli::ARCH_URL.to_string());

        let completion = args.filters.completion_percent.unwrap_or(100);

        let age = args.filters.age.unwrap_or(0);
        let rate = args.general.rate;
        let timeout = args.general.timeout;
        let include = args.general.include;

        Self::new(
            outfile,
            export,
            filters,
            view,
            sort,
            countries,
            ttl,
            url,
            args.filters.ipv4,
            args.filters.isos,
            args.filters.ipv6,
            completion,
            age,
            rate,
            timeout,
            include,
            args.general.direct,
        )
    }
}
