@id("alice-can-view-VacationPhoto94.jpg")
permit (
    principal == PhotoFlash::User::"alice",
    action == PhotoFlash::Action::"viewPhoto",
    resource == PhotoFlash::Photo::"VacationPhoto94.jpg"
)
when { context.authenticated == true };

@id("test-group-can-upload")
permit (
    principal is PhotoFlash::User in PhotoFlash::UserGroup::"test-group",
    action == PhotoFlash::Action::"uploadPhoto",
    resource == PhotoFlash::Album::"Vacation"
)
when { context.authenticated == true && context.photo.file_size < 10 };

@id("owner-can-upload")
permit (
    principal,
    action in
        [PhotoFlash::Action::"uploadPhoto", PhotoFlash::Action::"viewPhoto"],
    resource is PhotoFlash::Album
)
when { resource.account.owner == principal };

@id("anyone-can-view-public")
permit (
    principal,
    action == PhotoFlash::Action::"viewPhoto",
    resource is PhotoFlash::Photo
)
when { resource.private == false };