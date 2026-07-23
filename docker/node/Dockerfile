FROM node:latest

USER node

SHELL ["/bin/bash", "-c"]

RUN wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.bashrc" SHELL="$(which bash)" bash - \
    && source "/home/node/.bashrc"
