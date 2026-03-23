FROM alpine:latest
RUN apk add --no-cache iptables
RUN apk add --no-cache iproute2
RUN apk add --no-cache tcpdump
RUN apk conntrack-tools
RUN sysctl -w net.ipv4.ip_forward=1

RUN