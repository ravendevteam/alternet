start_all()
bootstrap.wait_for_unit("network.target")
bootstrap.wait_for_open_port(4001)