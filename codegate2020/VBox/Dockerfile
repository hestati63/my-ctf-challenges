FROM ubuntu:20.04

# Setup environ
ENV user vbox
ENV prob_port 1000
ENV DEBIAN_FRONTEND noninteractive

# Setup kvm.
ARG KVM_ROOT
ENV KVM_HOST=$KVM_ROOT
RUN addgroup --system --gid $KVM_HOST kvm

# Install packages
RUN sed -i "s/http:\/\/archive.ubuntu.com/http:\/\/kr.archive.ubuntu.com/g" /etc/apt/sources.list
RUN apt-get update
RUN apt-get install -y xinetd python3 qemu-kvm libvirt-daemon-system \
        libvirt-clients bridge-utils xinetd

RUN chmod 1733 /tmp /var/tmp /dev/shm

# Additional configuration
RUN adduser --disabled-login --gecos '' --force-badname $user
RUN adduser $user kvm
RUN adduser $user libvirt

ADD ./binary_flag/flag /home/$user/flag
ADD ./binary_flag/vbox /home/$user/vbox
ADD ./binary_flag/run.sh /home/$user/run.sh
ADD ./binary_flag/glue.py /home/$user/glue.py
ADD ./xinetd.conf /etc/xinetd.d/chal

RUN chown -R root:root /home/$user/
RUN chown root:$user /home/$user/vbox
RUN chown root:$user /home/$user/flag
RUN chown root:$user /home/$user/run.sh
RUN chown root:$user /home/$user/glue.py

RUN chmod 2755 /home/$user/vbox
RUN chmod 440 /home/$user/flag
RUN chmod 2755 /home/$user/glue.py

RUN chmod 2755 /home/$user/run.sh

EXPOSE $prob_port
RUN service xinetd restart
CMD ["/usr/sbin/xinetd", "-dontfork"]
