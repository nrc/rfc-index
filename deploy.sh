cargo run -- generate
cd target/out
sftp -b ../../deploy.sftp root@161.35.234.130
