FROM nvcr.io/nvidia/deepstream:6.2-devel

RUN apt update && \
    /opt/nvidia/deepstream/deepstream/user_additional_install.sh && \
    apt-get install --no-install-recommends -y \
        apt-transport-https \
        autoconf \
        automake \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        g++ \
        gir1.2-gst-rtsp-server-1.0 \
        git \
        gobject-introspection \
        graphviz \
        gstreamer1.0-libav \
        gstreamer1.0-plugins-bad \
        gstreamer1.0-plugins-base \
        gstreamer1.0-plugins-good \
        gstreamer1.0-plugins-ugly \
        gstreamer1.0-rtsp \
        libbz2-dev \
        libcairo2-dev \
        libffi-dev \
        libges-1.0-dev \
        libgirepository1.0-dev \
        libglib2.0-dev \
        libglib2.0-dev-bin \
        libgstreamer-plugins-base1.0-dev \
        libgstreamer1.0-dev \
        libgstreamer1.0-dev \
        libgstrtspserver-1.0-0 \
        libgstrtspserver-1.0-dev \
        libgstrtspserver-1.0-dev \
        liblzma-dev \
        libmagickwand-dev \
        libncurses5-dev \
        libncursesw5-dev \
        libpq-dev \
        libreadline-dev \
        libsqlite3-dev \
        libssl-dev \
        libtool \
        llvm \
        llvm-dev \
        libclang-dev \
        m4 \
        make \
        tk-dev \
        wget \
        xz-utils \
        zip \
        zlib1g-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN echo " \
    set -o vi \
    alias vi='vim' \
    " >> /root/.bashrc
