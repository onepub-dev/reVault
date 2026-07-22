Name:           revault-api
Version:        0.2.0
Release:        1%{?dist}
Summary:        Stable native API for reVault lockboxes and vaults
License:        LicenseRef-reVault-Source-Available-1.0
URL:            https://github.com/onepub-dev/reVault
Source0:        revault-api-native-linux-%{_arch}-gnu-%{version}.tar.gz
Requires:       dbus-libs

%description
Native ABI used by reVault language bindings.

%package devel
Summary:        Development files for revault-api
Requires:       %{name}%{?_isa} = %{version}-%{release}

%description devel
Header and linker entry point for the reVault native API.

%install
mkdir -p %{buildroot}%{_libdir} %{buildroot}%{_includedir}
install -m 0755 lib/librevault_api.so %{buildroot}%{_libdir}/librevault_api.so.1
ln -s librevault_api.so.1 %{buildroot}%{_libdir}/librevault_api.so
install -m 0644 include/revault_api.h %{buildroot}%{_includedir}/revault_api.h

%files
%license LICENSE
%{_libdir}/librevault_api.so.1

%files devel
%{_libdir}/librevault_api.so
%{_includedir}/revault_api.h
